use std::sync::Arc;
use tracing::error;
use gearbot_2_lib::kafka::message::InteractionCommand;
use crate::util::bot_context::BotContext;
use crate::util::error::InteractionError;

mod debug;

pub type InteractionResult = Result<(), InteractionError>;

pub async fn handle(token: String, command: InteractionCommand, context: Arc<BotContext>) {
    let result = match &command {
        InteractionCommand::Debug { component, guild_id } => debug::run(component, guild_id, &token, &context).await
    };

    if let Err(error) = result {
        if !error.is_user_error() {
            error!("Failed to handle interaction command: {} (interaction data: {:?})", error.get_log_error(), &command);
        }
        if let Err(e) = context.client.create_followup_message(&token).unwrap()
            //TODO: use actual lang
            .content(&error.get_user_error(&context.translator, "en_US"))
            .ephemeral(true)
            .exec()
            .await {
            error!("Failed to inform user of this failure! {}", e)
        }
    }
}