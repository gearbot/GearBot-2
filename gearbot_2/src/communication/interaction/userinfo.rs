use crate::communication::interaction::InteractionResult;
use crate::BotContext;
use gearbot_2_lib::translations::GearBotLangKey;
use gearbot_2_lib::util::error::GearError;
use gearbot_2_lib::util::url::{assemble_guild_avatar_url, assemble_user_avatar};
use gearbot_2_lib::util::{formatted_snowflake_timestamp, snowflake_age};
use std::sync::Arc;
use twilight_embed_builder::{EmbedAuthorBuilder, EmbedBuilder, ImageSource};
use twilight_model::id::{GuildId, UserId};

pub async fn run(user_id: u64, guild_id: u64, token: &str, context: &Arc<BotContext>) -> InteractionResult {
    let guild_id = GuildId::new(guild_id).unwrap();
    let user_id = UserId::new(user_id).unwrap();

    let member = context.get_guild_member(&guild_id, &user_id).await?;
    let mut user = member.as_ref().map_or(None, |member| Some(member.user()));

    // if we don't have a user ask the api for it
    if user.is_none() {
        user = context.get_user_info(&user_id).await?;
    }

    if let Some(user) = user {
        let mut builder = EmbedBuilder::new();

        let user_avatar = assemble_user_avatar(&user_id, user.discriminator, user.avatar.as_ref());
        let big_avatar = member.as_ref().map(|member| member.avatar.clone()).map_or_else(
            || user_avatar.clone(),
            |avatar| {
                avatar.map_or_else(
                    || user_avatar.clone(),
                    |avatar| assemble_guild_avatar_url(&guild_id, &user_id, avatar.as_ref()),
                )
            },
        );

        //TODO: actual locale
        let locale = "en_US";
        builder = builder
            .author(
                EmbedAuthorBuilder::new(
                    member
                        .as_ref()
                        .map_or_else(|| user.to_string(), |member| member.to_string()),
                )
                .icon_url(ImageSource::url(user_avatar)?),
            )
            .thumbnail(ImageSource::url(big_avatar)?)
            .description(
                context
                    .translator
                    .translate(locale, GearBotLangKey::UserinfoUser)
                    .arg("id", user_id.get())
                    .arg("created_on", formatted_snowflake_timestamp(&user_id))
                    .arg("age", snowflake_age(&user_id, 2, locale, &context.translator))
                    .build(),
            );

        context
            .client
            .create_followup_message(token)
            .unwrap()
            .embeds(&[builder.build()?])
            .exec()
            .await?;
    } else {
        return Err(GearError::UnknownUser(user_id));
    }

    Ok(())
}
