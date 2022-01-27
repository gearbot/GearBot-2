use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::{debug, error, info, trace, warn};
use twilight_model::gateway::payload::incoming::{GuildDelete, MemberChunk};
use twilight_model::gateway::payload::outgoing::request_guild_members::RequestGuildMembersBuilder;
use twilight_model::guild::{Guild as TwilightGuild, PartialGuild};

use gearbot_2_lib::kafka::message::{General, Message};
use gearbot_2_lib::kafka::sender::KafkaSender;
use gearbot_2_lib::util::markers::GuildId;

use crate::cache::guild::GuildCacheState;
use crate::cache::{Guild, Member};
use crate::util::bot_context::Context;
use crate::{communication, BotStatus};

pub fn on_guild_create(shard: u64, guild: TwilightGuild, context: &Context) {
    let guild_id = guild.id;
    let new: Arc<Guild> = Arc::new(guild.into());
    context
        .cache
        .insert_guild(shard, guild_id, new.clone(), &context.metrics);

    tokio::spawn(new_guild_processor(shard, guild_id, new, context.clone()));
}

pub fn on_guild_update(guild: PartialGuild, context: &Context) {
    let id = guild.id;
    if let (Some(_old), Some(_new)) = context.cache.update_guild(guild.id, guild) {
        trace!("Updated a guild");
    } else {
        warn!("Received a guild update for a guild that wasn't cached: {}", id);
    }
}

pub fn on_guild_delete(shard: u64, event: GuildDelete, context: &Context) {
    let old = context
        .cache
        .remove_guild(shard, event.id, event.unavailable, &context.metrics);
    if old.is_some() {
        if event.unavailable {
            info!("Guild {} became unavailable", event.id)
        } else {
            info!("Removed from guild {}", event.id)
        }
    } else {
        warn!(
            "Received a guild delete event for a server that wasn't cached: {}",
            event.id
        );
    }
}

pub fn on_member_chunk(shard: u64, chunk: MemberChunk, context: &Context) {
    let member_count = chunk.members.len();
    trace!(
        "Received chunk {}/{} for guild {} with {} members",
        chunk.chunk_index + 1,
        chunk.chunk_count,
        chunk.guild_id,
        member_count
    );

    let last = chunk.chunk_count - 1 == chunk.chunk_index;
    let mut new_users = Vec::new();
    if let Some(guild) = context.cache.get_guild(&chunk.guild_id) {
        // pass members to guild for caching
        let inserted = guild.receive_members(
            // map the twilight member to (UserId, Arc<Member>) for the cache
            chunk.members.into_iter().map(|member| {
                let uid = member.user.id;
                let member = if let Some(user) = context.cache.get_user(&member.user.id) {
                    Member::assemble(member, user)
                } else {
                    let member = Member::convert_with_user(member, None);
                    new_users.push((uid, member.user()));
                    member
                };
                (uid, Arc::new(member))
            }),
            last,
            &context.metrics,
            shard,
        );

        let user_count = new_users.len();

        //insert the users themselves
        context.cache.insert_users(new_users);

        // update metrics
        context.metrics.members.add(inserted as i64);
        context.metrics.users.add(user_count as i64);

        //if this was the last chunk we need to request the next one
        if last {
            // Safety net: declare us non pending so if a bug in the twilight websocket rate limit handling
            // causes this shard to die, we can recover
            context
                .pending_chunks
                .get(&shard)
                .unwrap()
                .store(false, Ordering::SeqCst);
            tokio::spawn(request_next_guild(shard, context.clone()));
        }
    } else {
        error!(
            "Got a member chunk for guild {} but no such guild exists in the cache!",
            &chunk.guild_id
        );
    }
}

// async function actually process the new guild
async fn new_guild_processor(shard: u64, guild_id: GuildId, _guild: Arc<Guild>, context: Context) {
    // are chunks already pending for this shard?
    let atom = context.pending_chunks.get(&shard).unwrap();
    if atom.load(Ordering::SeqCst) {
        // yes, queue up instead
        context.add_requested_guild(&shard, guild_id);
    } else {
        // no, set pending and request our members
        atom.store(true, Ordering::SeqCst);
        request_guild_members(shard, guild_id, &context).await
    }

    //todo: actually process the new guild
}

async fn request_guild_members(shard: u64, guild_id: GuildId, context: &Context) {
    if context.is_status(BotStatus::Terminating) {
        info!("Cluster is terminating but guild members where requested, canceling all pending member requests!");
        context.clear_requested_guilds();
        return;
    }

    let s = context.cluster.shard(shard).unwrap();
    debug!("Requesting guild members for guild {} on shard {}", guild_id, shard);
    // let info = s.info().unwrap();
    // info!("Shard rate limit info before: (refill: {:?}, refill - now: {:?}, requests: {})", info.ratelimit_refill(), info.ratelimit_refill() - Instant::now(), info.ratelimit_requests());
    if let Err(e) = s
        .command(
            &RequestGuildMembersBuilder::new(guild_id)
                .presences(false)
                .query("", None),
        )
        .await
    {
        error!(
            "Failed to request guild members for guild {} on shard {}: {}",
            guild_id, shard, e
        );

        // sending the command failed, log the error and unblock guild requesting
        // TODO: find a way to queue the next one if needed without infinite recursion compile issues
        context
            .pending_chunks
            .get(&shard)
            .unwrap()
            .store(false, Ordering::SeqCst);
    }
    // let info = s.info().unwrap();
    // info!("Shard rate limit info after is {:?}, {}", info.ratelimit_refill(), info.ratelimit_requests());
}

pub async fn request_next_guild(shard: u64, context: Context) {
    if let Some(guild_id) = context.get_next_requested_guild(&shard) {
        context
            .pending_chunks
            .get(&shard)
            .unwrap()
            .store(true, Ordering::SeqCst);
        request_guild_members(shard, guild_id, &context).await;
    } else {
        context
            .pending_chunks
            .get(&shard)
            .unwrap()
            .store(false, Ordering::SeqCst);

        let mut unfinished_business = Vec::new();
        // verify all are actually done
        context.cache.for_each_guild(|guild_id, guild| {
            // cache is all guilds on the cluster, only look at the ones for this specific shard
            if (guild_id.get() >> 22) % context.cluster_info.total_shards == shard {
                let state = guild.cache_state();
                if state == GuildCacheState::Created || state == GuildCacheState::ReceivingMembers {
                    unfinished_business.push(*guild_id)
                }
            }
        });

        if unfinished_business.is_empty() {
            info!("No more guild member requests pending for shard {}!", shard);
            if !context.has_any_requested_guilds() {
                info!("All guilds across all shards are now cached");
                if context.is_status(BotStatus::Starting) {
                    context.set_status(BotStatus::Standby);
                    // 20 seconds should be plenty of time to ensure the current primary instance gets it, even in a failover scenario
                    // the uuid is ensure we don't shut down ourselves accidentally
                    //
                    let goal =
                        (SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(30)).as_millis();

                    if let Err(e) = KafkaSender::new()
                        .send(
                            &context.get_queue_topic(),
                            &Message::General(General::ShutdownAt {
                                time: goal,
                                uuid: context.uuid.as_u128(),
                            }),
                        )
                        .await
                    {
                        error!("Failed to send the shutdown time to the old instance: {}", e);
                    } else {
                        // the primary cluster was informed, recalculate time left. using saturating sub to avoid overflowing into the next century
                        let left = Duration::from_millis(
                            goal.saturating_sub(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis())
                                as u64,
                        );

                        info!(
                            "Ready to take over as primary instance in {} seconds",
                            left.as_secs_f32()
                        );
                        tokio::time::sleep(left).await;
                        if context.is_status(BotStatus::Standby) {
                            info!("Taking over as primary instance!");
                            context.set_status(BotStatus::Primary);
                            if let Err(e) = communication::initialize(context.clone()).await {
                                error!("Failed to connect to the queue: {}", e)
                            }
                        } else {
                            info!("Promotion time reached but we where already in primary instance mode.")
                        }
                    }
                }
            }
        } else {
            warn!(
                "No more guild member requests where pending, yet {} guild(s) are not fully cached, retrying...",
                unfinished_business.len()
            );
            let guild = unfinished_business.pop();
            if !unfinished_business.is_empty() {
                context.add_requested_guilds(&shard, unfinished_business);
            }
            context
                .pending_chunks
                .get(&shard)
                .unwrap()
                .store(true, Ordering::SeqCst);
            // safe to unwrap, due to the empty check we know there was something to pop
            request_guild_members(shard, guild.unwrap(), &context).await;
        }
    }
}
