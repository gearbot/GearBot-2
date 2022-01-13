use std::sync::Arc;

use tracing::{trace, warn};
use twilight_model::gateway::payload::incoming::{MemberRemove, MemberUpdate};
use twilight_model::guild::Member as TwilightMember;
use twilight_model::id::{GuildId, UserId};

use crate::cache::guild::GuildCacheState;
use crate::cache::{Member, User};
use crate::util::bot_context::BotContext;

pub fn on_member_add(member: TwilightMember, context: &Arc<BotContext>) {
    let user_id = member.user.id;
    trace!("Member {} joined {}", &user_id, &member.guild_id);
    if let Some(guild) = context.cache.get_guild(&member.guild_id) {
        let member = if let Some(user) = context.cache.get_user(&user_id) {
            Member::assemble(member, user)
        } else {
            let member = Member::convert_with_user(member, None);

            //new user, add them to the user cache
            context.cache.insert_user(user_id, member.user());
            context.metrics.users.inc();

            member
        };
        // in either case we need to add this as mutual guild
        member.add_mutual_guild();

        guild.insert_member(user_id, Arc::new(member));
        context.metrics.members.inc()
    } else {
        warn!("Got a member add event for an uncached guild: {}", member.guild_id);
    }
}

pub fn on_member_update(member_update: MemberUpdate, context: &Arc<BotContext>) {
    let user_id = member_update.user.id;
    let guild_id = member_update.guild_id;
    trace!("Member {} updated on guild {}", &user_id, &member_update.guild_id);
    if let Some(guild) = context.cache.get_guild(&member_update.guild_id) {
        // grab the user
        if let Some(old_member) = guild.get_member(&member_update.user.id) {
            //grab the member
            if let Some(old_user) = context.cache.get_user(&member_update.user.id) {
                // check if the user and member where updated
                let user_updated = old_user.is_updated(&member_update.user);
                let member_updated = old_member.is_updated(&member_update);

                // assemble user and member. a bit messy but we get a lot of these events
                // so best to avoid unneeded cloning of all this data
                if member_updated {
                    let new_member = if user_updated {
                        let member = Arc::new(Member::convert_update(member_update, Some(old_user.clone())));
                        handle_updated_user(user_id, old_user, &member.user(), context);
                        member
                    } else {
                        Arc::new(Member::convert_update(member_update, Some(old_user)))
                    };
                    guild.insert_member(user_id, new_member.clone());
                    let _new_user = new_member.user();
                    tokio::spawn(log_updated_member(
                        user_id,
                        guild_id,
                        old_member,
                        new_member,
                        context.clone(),
                    ));
                } else if user_updated {
                    let new_user = Arc::new(User::assemble(member_update.user, Some(old_user.clone())));
                    handle_updated_user(user_id, old_user, &new_user, context);
                }
            }
        } else {
            // missing member
            if let Some(user) = context.cache.get_user(&user_id) {
                guild.insert_member(user_id, Arc::new(Member::from_update(member_update, user)));
                context.metrics.members.inc();
            } else {
                // missing user as well
                let member = Arc::new(Member::convert_update(member_update, None));
                context.cache.insert_user(user_id, member.user());
                guild.insert_member(user_id, member);

                context.metrics.users.inc();
                context.metrics.members.inc();
            }
        }
    } else {
        warn!(
            "Got a member update event for an uncached guild: {}",
            member_update.guild_id
        );
    }
}

fn handle_updated_user(user_id: UserId, old_user: Arc<User>, new_user: &Arc<User>, context: &Arc<BotContext>) {
    //This is handled here and not in the cache itself to avoid having to loop over everything twice
    context.cache.insert_user(user_id, new_user.clone());

    let mut guild_list = Vec::new();

    context.cache.for_each_guild(|guild_id, guild| {
        if let Some(member) = guild.get_member(&user_id) {
            member.set_user(new_user.clone());
            guild_list.push(*guild_id)
        }
    });

    tokio::spawn(log_updated_user(
        user_id,
        old_user,
        new_user.clone(),
        guild_list,
        context.clone(),
    ));
}

async fn log_updated_user(
    _user_id: UserId,
    _old_user: Arc<User>,
    _new_user: Arc<User>,
    _guild_list: Vec<GuildId>,
    _context: Arc<BotContext>,
) {
}

async fn log_updated_member(
    _user_id: UserId,
    _guild_id: GuildId,
    _old_member: Arc<Member>,
    _new_member: Arc<Member>,
    _context: Arc<BotContext>,
) {
}

pub fn on_member_remove(member_remove: MemberRemove, context: &Arc<BotContext>) {
    trace!("User {} left {}", &member_remove.user.id, &member_remove.guild_id);

    if let Some(guild) = context.cache.get_guild(&member_remove.guild_id) {
        let old = guild.remove_member(&member_remove.user.id);
        if let Some(old) = old {
            // cleanup the user if this was the last mutual guild
            // we still have an arc to use but this purges the cached cache copy if needed
            if old.get_mutual_guilds() == 0 {
                context.cache.cleanup_user(&member_remove.user.id)
            }
        } else if guild.cache_state() == GuildCacheState::Cached {
            warn!(
                "Got a member remove event for a user ({}) that wasn't cached in that guild ({})",
                &member_remove.user.id, &member_remove.guild_id
            );
        }
        //TODO: handle
    } else {
        warn!(
            "Got a member remove event for an uncached guild: {}",
            member_remove.guild_id
        );
    }
}
