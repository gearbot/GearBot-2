use chrono::Utc;
use crate::util::CommandError;
use twilight_model::application::callback::InteractionResponse;
use twilight_model::application::interaction::ApplicationCommand;
use twilight_util::builder::CallbackDataBuilder;
use twilight_model::channel::message::MessageFlags;
use gearbot_2_lib::translations::GearBotLangKey;
use crate::interactions::command::{CommandResult, Reply};
use crate::State;

pub fn execute() -> CommandResult {
    Ok(
        Reply {
            response:
            InteractionResponse::DeferredChannelMessageWithSource(
                CallbackDataBuilder::new().flags(MessageFlags::EPHEMERAL).build()
            ),
            followup: true,
        }
    )
}

pub async fn async_followup(command: Box<ApplicationCommand>, state: &State) -> Result<(), CommandError> {
    let start = Utc::now();
    state.discord_client.create_followup_message(&command.token).unwrap()
        .content(&state.translator.translate("en_US", GearBotLangKey::PingCalculating).build())
        .exec()
        .await?;
    let after = Utc::now() - start;
    let milli = after.num_milliseconds();

    state.discord_client.update_interaction_original(&command.token).unwrap()
        .content(
            Some(
                &state.translator.translate("en_US", GearBotLangKey::PingCalculated)
                    .arg("latency", milli)
                    .build()
            )
        )?
        .exec()
        .await?;

    Ok(())
}