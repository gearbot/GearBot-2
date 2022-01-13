use std::sync::Arc;

use num_format::{Locale, ToFormattedString};
use twilight_embed_builder::EmbedBuilder;
use twilight_http::request::AttachmentFile;
use twilight_model::id::GuildId;

use crate::communication::interaction::InteractionResult;
use crate::util::bot_context::BotContext;
use crate::util::error::InteractionError;

// this is a debug command, no need to bother with translations
pub async fn run(component: &str, guild_id: &u64, token: &str, context: &Arc<BotContext>) -> InteractionResult {
    match component {
        "cache" => {
            let mut guilds = 0;
            let mut members = 0;
            let mut channels = 0;
            let mut emoji = 0;
            let mut roles = 0;
            let users = context.cache.get_user_count();

            context.cache.for_each_guild(|_, guild| {
                guilds += 1;
                members += guild.get_member_count();
                channels += guild.get_channel_count();
                emoji += guild.get_emoji_count();
                roles += guild.get_role_count()
            });
            //TODO: use actual locale later
            let locale = Locale::nl_BE;
            context.client.create_followup_message(token)
                .unwrap()
                .embeds(
                    &[EmbedBuilder::new()
                        .title("Cache statistics")
                        .description(
                            format!("**Guilds**: {}\n**Members**: {}\n**Channels**: {}\n**Emoji**: {}\n**Roles**: {}\n**Users**: {}",
                            guilds.to_formatted_string(&locale),
                            members.to_formatted_string(&locale),
                            channels.to_formatted_string(&locale),
                            emoji.to_formatted_string(&locale),
                            roles.to_formatted_string(&locale),
                            users.to_formatted_string(&locale))
                        )
                        .build()?]
                )
                .exec()
                .await?;
        }
        "guild_config_bot" => {
            let info = context.get_guild_info(&GuildId::new(*guild_id).unwrap()).await?;
            let bytes = serde_json::to_vec_pretty(&info.config)?;
            context
                .client
                .create_followup_message(token)
                .unwrap()
                .attach(&[AttachmentFile::from_bytes("config.json", &bytes)])
                .exec()
                .await?;
        }
        wrong => return Err(InteractionError::InvalidOption(wrong.to_string())),
    }
    Ok(())
}
