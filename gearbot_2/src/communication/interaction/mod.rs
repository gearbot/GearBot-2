use tracing::error;

use crate::util::bot_context::Context;
use gearbot_2_lib::kafka::message::InteractionCommand;
use gearbot_2_lib::util::GearResult;

mod debug;
mod userinfo;

pub type InteractionResult = GearResult<()>;

pub async fn handle(token: String, locale: String, command: InteractionCommand, context: Context) {
    let result = match &command {
        InteractionCommand::Debug { component, guild_id } => {
            debug::run(component, guild_id, &token, &locale, &context).await
        }
        InteractionCommand::Userinfo { user_id, guild_id } => {
            userinfo::run(*user_id, *guild_id, &token, &locale, &context).await
        }
    };

    if let Err(error) = result {
        if !error.is_user_error() {
            error!(
                "Failed to handle interaction command: {} (interaction data: {:?})",
                error.get_log_error(),
                &command
            );
        }
        if let Err(e) = context
            .interaction_client()
            .create_followup_message(&token)
            .content(&error.get_user_error(&context.translator, &locale))
            .unwrap()
            .ephemeral(true)
            .exec()
            .await
        {
            error!("Failed to inform user of this failure! {}", e)
        }
    }
}
