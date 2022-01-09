use std::sync::Arc;
use tracing::error;
use twilight_model::channel::{Channel as TwilightChannel};
use twilight_model::id::{ChannelId, GuildId};
use crate::BotContext;
use crate::cache::Channel;


pub fn on_channel_create(channel: TwilightChannel, context: &Arc<BotContext>) {
    if let Some(new) = cache_channel_create(channel, context) {}
}

pub fn cache_channel_create(channel: TwilightChannel, context: &Arc<BotContext>) -> Option<Arc<Channel>> {
    let channel_id = channel.id();
    if let TwilightChannel::Guild(guild_channel) = channel {
        if let Some(guild_id) = guild_channel.guild_id() {
            let new: Arc<Channel> = Arc::new(Channel::from_guild_channel(guild_channel));
            context.cache.insert_channel(guild_id, channel_id, new.clone());
            return Some(new);
        } else {
            error!("Received a guild channel without guild id from twilight: {}", channel_id);
        }
    }
    None
}

pub fn on_channel_delete(channel: TwilightChannel, context: &Arc<BotContext>) {
    if let TwilightChannel::Guild(guild_channel) = channel {
        if let Some(guild_id) = guild_channel.guild_id() {
            if let Some(old) = cache_channel_delete(&guild_id, &guild_channel.id(), context) {}
        } else {
            error!("Received a guild channel delete without a guild id!")
        }
    }

}

pub fn cache_channel_delete(guild_id: &GuildId, channel_id: &ChannelId, context: &Arc<BotContext>) -> Option<Arc<Channel>> {
    return context.cache.remove_channel(&guild_id, channel_id);
}

pub fn on_channel_update(channel: TwilightChannel, context: &Arc<BotContext>) {
    if let Some((old, new)) = cache_channel_update(channel, context) {}
}

pub fn cache_channel_update(channel: TwilightChannel, context: &Arc<BotContext>) -> Option<(Option<Arc<Channel>>, Arc<Channel>)> {
    let channel_id = channel.id();
    if let TwilightChannel::Guild(guild_channel) = channel {
        if let Some(guild_id) = guild_channel.guild_id() {
            let new: Arc<Channel> = Arc::new(Channel::from_guild_channel(guild_channel));
            let old = context.cache.insert_channel(guild_id, channel_id, new.clone());
            return Some((old, new));
        } else {
            error!("Received a guild channel without guild id from twilight: {}", channel_id);
        }
    }
    None
}