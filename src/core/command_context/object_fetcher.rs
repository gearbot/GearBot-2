use std::sync::Arc;

use twilight_model::{
    channel::Message,
    guild::{Ban, Permissions},
    id::{ChannelId, MessageId, RoleId, UserId},
};

use super::CommandContext;
use crate::cache::{CachedChannel, CachedMember, CachedRole, CachedUser};
use crate::error::{CommandError, OtherFailure, ParseError};

impl CommandContext {
    pub async fn get_user(&self, user_id: UserId) -> Result<Arc<CachedUser>, CommandError> {
        self.bot_context
            .get_user(user_id)
            .await
            .map_err(CommandError::ParseError)
    }

    pub async fn get_member(&self, user_id: &UserId) -> Option<Arc<CachedMember>> {
        self.bot_context.cache.get_member(&self.guild.id, user_id).await
    }

    pub async fn get_channel(&self, channel_id: ChannelId) -> Option<Arc<CachedChannel>> {
        self.bot_context.cache.get_channel(channel_id).await
    }

    pub async fn get_role(&self, role_id: &RoleId) -> Option<Arc<CachedRole>> {
        match self.guild.get_role(role_id).await {
            Some(guard) => Some(guard),
            None => None,
        }
    }

    pub async fn get_ban(&self, user_id: UserId) -> Result<Option<Ban>, CommandError> {
        Ok(self.bot_context.http.ban(self.guild.id, user_id).await?)
    }

    pub async fn get_dm_for_author(&self) -> Result<Arc<CachedChannel>, twilight_http::Error> {
        self.get_dm_for_user(self.message.author.id).await
    }

    //TODO: still usable with api v8 since we do not get a channel create event?
    pub async fn get_dm_for_user(&self, user_id: UserId) -> Result<Arc<CachedChannel>, twilight_http::Error> {
        match self.bot_context.cache.get_dm_channel_for(user_id).await {
            Some(channel) => Ok(channel),
            None => {
                let channel = self.bot_context.http.create_private_channel(user_id).await?;
                Ok(self.bot_context.cache.insert_private_channel(&channel).await)
            }
        }
    }

    pub async fn get_message(&mut self) -> Result<Message, CommandError> {
        let input = self.parser.get_next()?;

        let user_id = self.message.author.id;
        let message_id = input.parse::<u64>().map_err(|_| CommandError::NoDM)?;
        let channel_id = self.message.channel.get_id();

        let channel = match self.bot_context.cache.get_channel(channel_id).await {
            Some(ch) => ch,
            None => return Err(CommandError::ParseError(ParseError::UnknownChannel(channel_id.0))),
        };

        if let CachedChannel::TextChannel { name, id, .. } = &*channel {
            let bot_has_access = self
                .bot_has_channel_permissions(Permissions::VIEW_CHANNEL & Permissions::READ_MESSAGE_HISTORY)
                .await;

            // Verify that the bot has access
            if !bot_has_access {
                return Err(CommandError::ParseError(ParseError::NoChannelAccessBot(name.clone())));
            }

            let user_has_access = self
                .has_channel_permissions(
                    user_id,
                    *id,
                    Permissions::VIEW_CHANNEL & Permissions::READ_MESSAGE_HISTORY,
                )
                .await;

            // Verify that the user has access
            if !user_has_access {
                return Err(ParseError::NoChannelAccessUser(name.clone()).into());
            }

            // All good, fetch the message from the api instead of cache to make sure it's not only up to date but still actually exists
            let result = self.bot_context.http.message(*id, MessageId(message_id)).await;

            match result {
                Ok(message) => Ok(message.unwrap()),
                Err(error) => {
                    if error.to_string().contains("status: 404") {
                        Err(CommandError::ParseError(ParseError::UnknownMessage))
                    } else {
                        Err(CommandError::OtherFailure(OtherFailure::TwilightHttp(error)))
                    }
                }
            }
        } else {
            unreachable!()
        }
    }
}
