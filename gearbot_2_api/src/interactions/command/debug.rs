use std::sync::Arc;
use twilight_model::application::interaction::ApplicationCommand;
use gearbot_2_lib::kafka::message::{InteractionCommand, Message};
use crate::interactions::command::get_required_string_value;
use crate::State;
use crate::util::CommandError;

pub async fn async_followup(command: Box<ApplicationCommand>, state: &Arc<State>) -> Result<(), CommandError> {
    state.kafka_sender.send(
        "gearbot_cluster_0",
        &Message::new_interaction(command.token, InteractionCommand::Debug {component: get_required_string_value("component", &command.data.options)?.to_string()}),
    ).await?;
    Ok(())
}