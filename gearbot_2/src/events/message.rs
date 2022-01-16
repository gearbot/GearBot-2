use crate::BotContext;
use gearbot_2_lib::datastore::guild::GuildDatastore;
use gearbot_2_lib::util::GearResult;
use std::sync::Arc;
use tracing::info;
use twilight_model::gateway::payload::incoming::{MessageCreate, MessageUpdate};

pub async fn on_message(message: MessageCreate, context: Arc<BotContext>) -> GearResult<()> {
    // we don't care about dms
    if let Some(guild_id) = &message.guild_id {
        let info = context.get_guild_info(guild_id).await?;

        // do we want messages logged for this guild?
        if !info.config.message_logs.enabled {
            return Ok(());
        }

        let datastore = GuildDatastore::new(&context.datastore, &info.encryption_key, guild_id);
        datastore
            .store_message(
                &message.id,
                &message.content,
                &message.author.id,
                &message.channel_id,
                &message.sticker_items,
                message.kind,
                message.attachments.len() as i32,
                message.pinned,
            )
            .await?;

        //todo: attachment metadata
    }

    Ok(())
}

pub async fn on_message_update(update: MessageUpdate, context: Arc<BotContext>) -> GearResult<()> {
    if let Some(guild_id) = &update.guild_id {
        let info = context.get_guild_info(guild_id).await?;

        // do we want messages logged for this guild?
        if !info.config.message_logs.enabled {
            return Ok(());
        }

        let content = update.content.unwrap_or_else(|| "".to_string());

        let datastore = GuildDatastore::new(&context.datastore, &info.encryption_key, guild_id);
        if let Some(old) = datastore
            .update_message(
                &update.id,
                &content,
                update.pinned.unwrap_or(false),
                update.attachments.map_or(0, |list| list.len() as i32),
            )
            .await?
        {
            info!("Message updated {} => {}", old.content, content);
        }
    }

    Ok(())
}
