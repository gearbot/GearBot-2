use num_format::{Locale, ToFormattedString};
use twilight_embed_builder::EmbedBuilder;
use twilight_http::request::AttachmentFile;

use gearbot_2_lib::util::error::GearError;
use gearbot_2_lib::util::markers::GuildId;

use crate::communication::interaction::InteractionResult;
use crate::util::bot_context::Context;

// this is a debug command, no need to bother with translations
pub async fn run(component: &str, guild_id: &u64, token: &str, locale: &str, context: &Context) -> InteractionResult {
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
            let locale = Locale::from_name(locale).unwrap_or(Locale::en_US_POSIX);
            context.interaction_client().create_followup_message(token)
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
                )?
                .exec()
                .await?;
        }
        "guild_config_bot" => {
            let info = context.get_guild_info(&GuildId::new(*guild_id)).await?;
            let bytes = serde_json::to_vec_pretty(&info.config)?;
            context
                .interaction_client()
                .create_followup_message(token)
                .attach(&[AttachmentFile::from_bytes("config.json", &bytes)])
                .exec()
                .await?;
        }
        wrong => return Err(GearError::InvalidOption(wrong.to_string())),
    }
    Ok(())
}
