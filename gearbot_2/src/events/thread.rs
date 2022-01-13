use std::sync::Arc;
use tracing::warn;
use crate::events::channel::{cache_channel_create, cache_channel_delete, cache_channel_update};
use twilight_model::channel::Channel as TwilightChannel;
use twilight_model::gateway::payload::incoming::{ThreadDelete, ThreadListSync, ThreadMembersUpdate};
use crate::util::bot_context::BotContext;

pub fn on_thread_create(channel: TwilightChannel, context: &Arc<BotContext>) {
    if let Some(new) = cache_channel_create(channel, context) {

    }
}

pub fn on_thread_delete(thread_delete: ThreadDelete, context: &Arc<BotContext>) {
    if let Some(old) = cache_channel_delete(&thread_delete.guild_id, &thread_delete.id, context) {

    }
}

pub fn on_thread_update(channel: TwilightChannel, context: &Arc<BotContext>)  {
    if let Some((old, new)) = cache_channel_update(channel, context) {

    }
}

pub fn on_thread_sync(sync: ThreadListSync, context: &Arc<BotContext>) {
    if let Some(guild) = context.cache.get_guild(&sync.guild_id) {
        guild.thread_sync(sync)
    } else {
        warn!("Received a guild thread sync for an uncached guild!")
    }
}

pub fn on_thread_members_update(update: ThreadMembersUpdate, context: &Arc<BotContext>) {

}