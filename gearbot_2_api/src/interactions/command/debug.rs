use std::sync::Arc;

use twilight_http::request::AttachmentFile;
use twilight_model::application::interaction::ApplicationCommand;

use gearbot_2_lib::kafka::message::{InteractionCommand, Message};

use crate::interactions::command::get_required_string_value;
use crate::util::CommandError;
use crate::State;

pub async fn async_followup(command: Box<ApplicationCommand>, state: &Arc<State>) -> Result<(), CommandError> {
    let component = get_required_string_value("component", &command.data.options)?.to_string();
    match component.as_str() {
        "guild_config" => {
            let info = state
                .datastore
                .get_or_create_guild_info(&command.guild_id.unwrap())
                .await?;
            let bytes = serde_json::to_vec_pretty(&info.config)?;
            state
                .discord_client
                .create_followup_message(&command.token)
                .unwrap()
                .attach(&[AttachmentFile::from_bytes("config.json", &bytes)])
                .exec()
                .await?;
        }
        _ => {
            state
                .kafka_sender
                .send(
                    "gearbot_cluster_0",
                    &Message::new_interaction(
                        command.token,
                        InteractionCommand::Debug {
                            component,
                            // safe to unwrap as it's a test command
                            guild_id: command.guild_id.unwrap().get(),
                        },
                    ),
                )
                .await?;
        }
    }

    Ok(())
}
