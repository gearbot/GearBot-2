use crate::interactions::command::get_required_user_id_value;
use crate::State;
use gearbot_2_lib::kafka::message::{InteractionCommand, Message};
use gearbot_2_lib::util::GearResult;
use std::sync::Arc;
use twilight_model::application::interaction::ApplicationCommand;

pub async fn async_followup(command: Box<ApplicationCommand>, state: &Arc<State>) -> GearResult<()> {
    let user = get_required_user_id_value("user", &command.data.options)?;

    // safe to unwrap as this is not usable in dms
    //todo: mark this command as guild only once the new permissions are out
    let guild_id = command.guild_id.unwrap();

    state
        .kafka_sender
        .send(
            &state.queue_for_guild(&guild_id),
            &Message::Interaction {
                token: command.token,
                command: InteractionCommand::Userinfo {
                    user_id: user.get(),
                    guild_id: guild_id.get(),
                },
            },
        )
        .await?;

    Ok(())
}
