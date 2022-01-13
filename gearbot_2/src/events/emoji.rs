use std::sync::Arc;

use twilight_model::gateway::payload::incoming::GuildEmojisUpdate;

use crate::util::bot_context::BotContext;

pub fn on_emoji_update(emoji_update: GuildEmojisUpdate, context: &Arc<BotContext>) {
    if let Some(guild) = context.cache.get_guild(&emoji_update.guild_id) {
        guild.update_emoji(emoji_update.emojis)
    }
}
