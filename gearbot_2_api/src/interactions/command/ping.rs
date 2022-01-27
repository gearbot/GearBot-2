use chrono::Utc;
use twilight_model::application::interaction::ApplicationCommand;

use gearbot_2_lib::translations::GearBotLangKey;
use gearbot_2_lib::util::GearResult;

use crate::State;

pub async fn async_followup(command: Box<ApplicationCommand>, state: &State) -> GearResult<()> {
    let start = Utc::now();
    state
        .interaction_client()
        .create_followup_message(&command.token)
        .content(
            &state
                .translator
                .translate("en_US", GearBotLangKey::PingCalculating)
                .build(),
        )?
        .exec()
        .await?;
    let after = Utc::now() - start;
    let milli = after.num_milliseconds();

    state
        .interaction_client()
        .update_interaction_original(&command.token)
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
