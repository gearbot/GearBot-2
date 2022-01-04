use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, trace};
use twilight_model::guild::{MfaLevel, NSFWLevel, PartialGuild, VerificationLevel};
use twilight_model::id::{ChannelId, EmojiId, GuildId, RoleId, UserId};
use crate::cache::{Channel, Emoji, Member, Role};
use twilight_model::channel::GuildChannel;
use twilight_model::gateway::payload::incoming::ThreadListSync;
use twilight_model::guild::Guild as TwilightGuild;
use twilight_model::guild::Emoji as TwilightEmoji;
use twilight_model::guild::Role as TwilightRole;
use twilight_model::voice::VoiceState as TwilightVoiceState;
use crate::{Cache, Metrics};
use crate::cache::voice_state::VoiceState;

#[derive(Clone, Eq, PartialEq)]
pub enum GuildCacheState {
    Created,
    ReceivingMembers,
    Cached,
    Unavailable,
}

impl GuildCacheState {
    pub fn name(&self) -> &str {
        match self {
            GuildCacheState::Created => "Created",
            GuildCacheState::ReceivingMembers => "Receiving members",
            GuildCacheState::Cached => "Cached",
            GuildCacheState::Unavailable => "Unavailable"
        }
    }
}

pub struct Guild {
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub owner: UserId,
    pub verification_level: VerificationLevel,
    roles: RwLock<HashMap<RoleId, Arc<Role>>>,
    emoji: RwLock<HashMap<EmojiId, Arc<Emoji>>>,
    pub features: Vec<String>,
    pub mfa: MfaLevel,
    channels: Arc<RwLock<HashMap<ChannelId, Arc<Channel>>>>,
    pub presence_limit: u64,
    pub max_members: u64,
    pub vanity_invite: Option<String>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub guild_locale: String,
    pub nsfw: NSFWLevel,
    members: Arc<RwLock<HashMap<UserId, Arc<Member>>>>,
    voice_states: Arc<RwLock<HashMap<UserId, Arc<VoiceState>>>>,

    cache_state: RwLock<GuildCacheState>,

}

impl From<TwilightGuild> for Guild {
    fn from(guild: TwilightGuild) -> Self {
        // guild.voice_states
        Guild {
            name: guild.name,
            icon: guild.icon,
            splash: guild.splash,
            owner: guild.owner_id,
            verification_level: guild.verification_level,
            roles: RwLock::new(convert_roles(guild.roles)),
            emoji: RwLock::new(convert_emoji(guild.emojis)),
            features: guild.features,
            mfa: guild.mfa_level,
            channels: Arc::new(RwLock::new(convert_channels(guild.channels))),
            presence_limit: 0,
            max_members: 0,
            vanity_invite: None,
            description: None,
            banner: None,
            guild_locale: "".to_string(),
            nsfw: NSFWLevel::Default,
            members: Default::default(),
            voice_states: Arc::new(RwLock::new(convert_voice_states(guild.voice_states))),
            cache_state: RwLock::new(GuildCacheState::Created),
        }
    }
}

impl Guild {
    pub fn update(old :&Arc<Guild>, new: PartialGuild) -> Self {
        Guild {
            name: new.name,
            icon: new.icon,
            splash: new.splash,
            owner: new.owner_id,
            verification_level: new.verification_level,
            roles: RwLock::new(convert_roles(new.roles)),
            emoji: RwLock::new(convert_emoji(new.emojis)),
            features: new.features,
            mfa: new.mfa_level,
            channels: old.channels.clone(),
            presence_limit: new.max_presences.unwrap_or_default(),
            max_members: new.max_members.unwrap_or_default(),
            vanity_invite: new.vanity_url_code,
            description: new.description,
            banner: new.banner,
            guild_locale: new.preferred_locale,
            nsfw: new.nsfw_level,
            members: old.members.clone(),
            voice_states: old.voice_states.clone(),
            cache_state: RwLock::new(old.cache_state.read().clone())
        }
    }

    pub fn insert_role(&self, role: Arc<Role>) -> Option<Arc<Role>>{
        self.roles.write().insert(role.id, role)
    }

    pub fn remove_role(&self, role_id: &RoleId) -> Option<Arc<Role>> {
        self.roles.write().remove(role_id)
    }

    // Bulk receiving members from member chunks, returns how many of them where new
    pub fn receive_members(&self, members: impl Iterator<Item=(UserId, Arc<Member>)>, last: bool, metrics: &Metrics, shard: u64) -> u64 {
        // store members and increase the user mutual guilds count
        let mut stored_members = self.members.write();
        let mut inserted = 0;
        for (member_id, member) in members {
            member.add_mutual_guild();
            if stored_members.insert(member_id, member).is_none() {
                inserted += 1;
            }
        }
        // update cache state
        let new_state = if last { GuildCacheState::Cached } else { GuildCacheState::ReceivingMembers };
        let mut state = self.cache_state.write();

        metrics.guilds.with_label_values(&[&shard.to_string(), state.name()]).dec();
        metrics.guilds.with_label_values(&[&shard.to_string(), new_state.name()]).inc();
        *state = new_state;

        inserted

    }

    pub fn insert_member(&self, user_id: UserId, member: Arc<Member>) -> Option<Arc<Member>> {
        //DO NOT increase mutual guilds count here since this could be a member update as well!!!
        self.members.write().insert(user_id, member)
    }

    pub fn remove_member(&self, user_id: &UserId) -> Option<Arc<Member>> {
        let member = self.members.write().remove(user_id);
        if let Some(member) = &member {
            member.remove_mutual_guild();
        }
        member
    }

    pub fn get_member(&self, user_id: &UserId) -> Option<Arc<Member>> {
        self.members.read().get(user_id).cloned()
    }

    pub fn insert_channel(&self, channel_id: ChannelId, channel: Arc<Channel>) -> Option<Arc<Channel>> {
        self.channels.write().insert(channel_id, channel)
    }

    pub fn remove_channel(&self, channel_id: &ChannelId) -> Option<Arc<Channel>> {
        self.channels.write().remove(channel_id)
    }

    pub fn cache_state(&self) -> GuildCacheState {
        self.cache_state.read().clone()
    }

    pub fn update_emoji(&self, emoji: Vec<TwilightEmoji>) {
        *self.emoji.write() = convert_emoji(emoji)
    }


    pub fn thread_sync(&self, sync: ThreadListSync) {
        let mut channels = self.channels.write();
        // channels.retain(|_, channel| channel.parent_id.map_or_else(|| true, |parent_id| sync.channel_ids.contains(&parent_id) ));
        todo!("Needs twilight type fix to be released")
    }

    pub fn set_voice_state(&self, user_id: UserId, state: Option<Arc<VoiceState>>) -> Option<Arc<VoiceState>> {
        let mut states = self.voice_states.write();
        if let Some(state) = state {
            states.insert(user_id, state)
        } else {
            states.remove(&user_id)
        }
    }
}

fn convert_emoji(raw_emoji: Vec<TwilightEmoji>) -> HashMap<EmojiId, Arc<Emoji>> {
    let mut emojis = HashMap::with_capacity(raw_emoji.len());
    for emoji in raw_emoji {
        emojis.insert(emoji.id, Arc::new(Emoji::from_emoji(emoji)));
    }
    emojis
}

fn convert_roles(raw_roles: Vec<TwilightRole>) -> HashMap<RoleId, Arc<Role>> {
    let mut roles = HashMap::with_capacity(raw_roles.len());
    for role in raw_roles {
        roles.insert(role.id, Arc::new(Role::from_role(role)));
    }
    roles
}

fn convert_channels(raw_channels: Vec<GuildChannel>) -> HashMap<ChannelId, Arc<Channel>> {
    let mut channels = HashMap::with_capacity(raw_channels.len());
    for channel in raw_channels {
        channels.insert(channel.id(), Arc::new(Channel::from_guild_channel(channel)));
    }
    channels
}

fn convert_voice_states(raw_states: Vec<TwilightVoiceState>) -> HashMap<UserId, Arc<VoiceState>> {
    let mut voice_states = HashMap::with_capacity(raw_states.len());
    for state in raw_states {
        let uid = state.user_id;
        if let Some(s) = VoiceState::from_state(state) {
            voice_states.insert(uid, Arc::new(s));
        }
    }
    voice_states
}


impl Cache {
    // Insert a new guild state and return the previous one if one exists.
    pub fn insert_guild(&self, shard: u64, guild_id: GuildId, new: Arc<Guild>, metrics: &Metrics) -> Option<Arc<Guild>> {
        trace!("Inserting guild {} into the cache", guild_id);

        // cleanup unavailable metric if needed
        if let Some(index) = self.unavailable_guilds.read().iter().position(|id| id == &guild_id) {
            self.unavailable_guilds.write().remove(index);
            metrics.guilds.get_metric_with_label_values(&[&shard.to_string(), GuildCacheState::Unavailable.name()]).unwrap().dec();
        }

        // Insert the new guild
        let old = self.guilds.write().insert(guild_id, new);

        if let Some(old_guild) = &old {
            //One already existed, cleanup metrics and user cache
            self.cleanup_guild(shard, old_guild, metrics);
        }

        metrics.guilds.get_metric_with_label_values(&[&shard.to_string(), GuildCacheState::Created.name()]).unwrap().inc();


        old
    }

    pub fn update_guild(&self, guild_id: GuildId, guild: PartialGuild) -> (Option<Arc<Guild>>, Option<Arc<Guild>>) {
        let mut guilds = self.guilds.write();
        let old = guilds.remove(&guild_id);

        // migrate the channels, members and state
        if let Some(old_guild) = &old {
            let new = Guild::update(old_guild, guild);
            // Arc it up and store it
            let new = Arc::new(new);

            guilds.insert(guild_id, new.clone());
            (old, Some(new))
        } else {
            (None, None)
        }


    }

    pub fn remove_guild(&self, shard: u64, guild_id: GuildId, unavailable: bool, metrics: &Metrics) -> Option<Arc<Guild>> {
        let old = self.guilds.write().remove(&guild_id);

        if let Some(guild) = &old {
            self.cleanup_guild(shard, guild, metrics)
        }

        if unavailable {
            self.unavailable_guilds.write().push(guild_id);
            metrics.guilds.get_metric_with_label_values(&[&shard.to_string(), GuildCacheState::Unavailable.name()]).unwrap().inc();
        }


        old
    }


    /// Cleanup global user cache after guild removal.
    /// Also remove it from the guild cache state counter, the caller is responsible for increasing
    /// the counter for the new status if applicable
    fn cleanup_guild(&self, shard: u64, guild: &Arc<Guild>, metrics: &Metrics) {
        let len = guild.members.read().len();
        if len > 0 {
            // We had users cached, reduce their mutual counts and purge those with no mutual guilds left
            let mut to_purge = Vec::new();
            let members = guild.members.read();
            for (user_id, member) in members.iter() {
                let old_count = member.remove_mutual_guild();
                if old_count == 1 {
                    to_purge.push(user_id)
                }
            }

            // actually purge these users, inner scope for write lock release
            {
                let mut users = self.users.write();
                for user in &to_purge {
                    users.remove(user);
                }
                debug!("Purged {} users from cache, {} remain", to_purge.len(), users.len());
            }

            metrics.members.sub(guild.members.read().len() as i64);
            metrics.users.sub(to_purge.len() as i64);
        }
        metrics.guilds.get_metric_with_label_values(&[&shard.to_string(), guild.cache_state.read().name()]).unwrap().dec();
    }

    pub fn get_guild(&self, guild_id: &GuildId) -> Option<Arc<Guild>> {
        self.guilds.read().get(guild_id).cloned()
    }
}