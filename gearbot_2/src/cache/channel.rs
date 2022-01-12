use crate::Cache;
use std::sync::Arc;
use tracing::error;
use twilight_model::channel::permission_overwrite::PermissionOverwrite;
use twilight_model::channel::thread::ThreadMetadata;
use twilight_model::channel::{ChannelType, GuildChannel};
use twilight_model::id::{ChannelId, GuildId};

pub struct Channel {
    pub channel_type: ChannelType,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub name: String,
    pub topic: Option<String>,
    pub nsfw: bool,
    pub bitrate: u64,
    pub user_limit: u64,
    pub user_rate_limit: u64,
    pub parent_id: Option<ChannelId>,
    pub thread_meta: Option<ThreadMetadata>,
}

impl Channel {
    pub fn from_guild_channel(channel: GuildChannel) -> Self {
        match channel {
            GuildChannel::Category(category) => Channel {
                channel_type: ChannelType::GuildCategory,
                permission_overwrites: category.permission_overwrites,
                name: category.name,
                topic: None,
                nsfw: false,
                bitrate: 0,
                user_limit: 0,
                user_rate_limit: 0,
                parent_id: None,
                thread_meta: None,
            },
            GuildChannel::NewsThread(thread) => Channel {
                channel_type: ChannelType::GuildPrivateThread,
                permission_overwrites: Vec::new(),
                name: thread.name,
                topic: None,
                nsfw: false,
                bitrate: 0,
                user_limit: 0,
                user_rate_limit: 0,
                parent_id: thread.parent_id,
                thread_meta: Some(thread.thread_metadata),
            },
            GuildChannel::PrivateThread(thread) => Channel {
                channel_type: ChannelType::GuildPrivateThread,
                permission_overwrites: Vec::new(),
                name: thread.name,
                topic: None,
                nsfw: false,
                bitrate: 0,
                user_limit: 0,
                user_rate_limit: 0,
                parent_id: thread.parent_id,
                thread_meta: Some(thread.thread_metadata),
            },
            GuildChannel::PublicThread(thread) => Channel {
                channel_type: ChannelType::GuildPrivateThread,
                permission_overwrites: Vec::new(),
                name: thread.name,
                topic: None,
                nsfw: false,
                bitrate: 0,
                user_limit: 0,
                user_rate_limit: 0,
                parent_id: thread.parent_id,
                thread_meta: Some(thread.thread_metadata),
            },
            GuildChannel::Text(channel) => Channel {
                channel_type: ChannelType::GuildText,
                permission_overwrites: channel.permission_overwrites,
                name: channel.name,
                topic: channel.topic,
                nsfw: channel.nsfw,
                bitrate: 0,
                user_limit: 0,
                user_rate_limit: channel.rate_limit_per_user.unwrap_or_default(),
                parent_id: channel.parent_id,
                thread_meta: None,
            },
            GuildChannel::Voice(channel) => Channel {
                channel_type: ChannelType::GuildVoice,
                permission_overwrites: channel.permission_overwrites,
                name: channel.name,
                topic: None,
                nsfw: false,
                bitrate: channel.bitrate,
                user_limit: channel.user_limit.unwrap_or_default(),
                user_rate_limit: 0,
                parent_id: channel.parent_id,
                thread_meta: None,
            },
            GuildChannel::Stage(stage) => Channel {
                channel_type: ChannelType::GuildStageVoice,
                permission_overwrites: stage.permission_overwrites,
                name: stage.name,
                topic: None,
                nsfw: false,
                bitrate: stage.bitrate,
                user_limit: stage.user_limit.unwrap_or_default(),
                user_rate_limit: 0,
                parent_id: stage.parent_id,
                thread_meta: None,
            },
        }
    }
}

impl Cache {
    pub fn insert_channel(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        channel: Arc<Channel>,
    ) -> Option<Arc<Channel>> {
        if let Some(guild) = self.guilds.read().get(&guild_id) {
            guild.insert_channel(channel_id, channel)
        } else {
            error!("Tried to add a channel to a guild that isn't cached: {}", guild_id);
            None
        }
    }

    pub fn remove_channel(&self, guild_id: &GuildId, channel_id: &ChannelId) -> Option<Arc<Channel>> {
        if let Some(guild) = self.guilds.read().get(guild_id) {
            guild.remove_channel(channel_id)
        } else {
            error!("Tried to remove a channel from a guild that isn't cached: {}", guild_id);
            None
        }
    }
}
