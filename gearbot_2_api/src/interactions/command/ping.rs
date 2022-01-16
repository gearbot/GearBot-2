use crate::State;
use chrono::Utc;
use gearbot_2_lib::translations::GearBotLangKey;
use gearbot_2_lib::util::GearResult;
use twilight_model::application::interaction::ApplicationCommand;

pub async fn async_followup(command: Box<ApplicationCommand>, state: &State) -> GearResult<()> {
    let start = Utc::now();
    state
        .discord_client
        .create_followup_message(&command.token)
        .unwrap()
        .content(
            &state
                .translator
                .translate("en_US", GearBotLangKey::PingCalculating)
                .build(),
        )
        .exec()
        .await?;
    let after = Utc::now() - start;
    let milli = after.num_milliseconds();

    state
        .discord_client
        .update_interaction_original(&command.token)
        .unwrap()
        .content(Some(
            &state
                .translator
                .translate("en_US", GearBotLangKey::PingCalculated)
                .arg("latency", milli)
                .build(),
        ))?
        .exec()
        .await?;

    Ok(())
}
