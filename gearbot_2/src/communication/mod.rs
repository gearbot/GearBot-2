use std::sync::Arc;
use bincode::config::Configuration;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message as AnnoyingConflict;
use tracing::{debug, error, info, trace, warn};
use gearbot_2_lib::kafka::listener::new_listener;
use gearbot_2_lib::kafka::message::{General, Message};
use gearbot_2_lib::kafka::sender::{KafkaSender, KafkaSenderError};
use crate::BotContext;
use crate::util::BotStatus;

mod general;
mod interaction;


// we only need one so fine to crash out on startup
pub async fn initialize(context: Arc<BotContext>) -> Result<(), KafkaSenderError> {
    info!("Initializing kafka queue communication...");
    let topic = format!("{}_cluster_{}", context.cluster_info.cluster_identifier, context.cluster_info.cluster_id);

    // first we send a no-op "Hello" message to the queue, this serves multiple purposes
    //1. we ensure the queue exists, if not the broker will make it
    //2. if there is no primary instance running for this cluster, we will be the ones receiving our own message, this will trigger immediate promotion to primary instance mode.

    debug!("Greeting the cluster queue...");
    KafkaSender::new().send(&topic, &Message::General(General::Hello)).await?;

    match new_listener(&[&topic], Some(&context.cluster_info.cluster_identifier)) {
        Ok(listener) => {
            info!("Communication link ready. Spawning background task to receive messages on topic '{}'", topic);
            tokio::spawn(receiver(listener, context));
            Ok(())
        }
        Err(e) => {
            Err(KafkaSenderError::Kafka(e))
        }
    }
}

async fn receiver(listener: StreamConsumer, context: Arc<BotContext>) {
    loop {
        let message = listener.recv().await;
        // check bot status so we can move into primary instance mode if needed
        let status = context.status();
        if matches!(status, BotStatus::STARTING | BotStatus::STANDBY) {
            info!("Got a cluster command while in {} mode, promoting to primary instance mode.", status.name());
            context.set_status(BotStatus::PRIMARY);
        }

        match message {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    match bincode::decode_from_slice::<Message, Configuration>(payload, Configuration::standard()) {
                        Ok((decoded_message, _)) => {
                            trace!("Received message on topic {}: {:?}", message.topic(), decoded_message);

                            // commit the message, this tells the server we processed this message (and all the ones before it)
                            // we do this explicitly to get predictable and dependable 'at most once' delivery behavior
                            // since double processing a message can be dangerous and confusing to the user
                            if let Err(e) = listener.commit_message(&message, CommitMode::Async) {
                                error!("Failed to commit queue index! {}", e)
                            }
                            handle_message(decoded_message, context.clone());


                            // terminate if needed
                            if matches!(context.status(), BotStatus::TERMINATING) {
                                info!("Kafka queue message receiver terminated!");
                                return;
                            }

                        }
                        Err(e) => {
                            error!("Failed to decode message on topic {}: {}", message.topic(), e)
                        }
                    }
                } else {
                    warn!("Received an empty message on topic {}!", message.topic())
                }
            }
            Err(e) => {
                error!("Failed to receive message from kafka queue: {}", e)
            }
        }
    }
}

fn handle_message(message: Message, context: Arc<BotContext>) {
    match message {
        Message::General(message) =>
            general::handle(message, context),
        Message::Interaction { token, command } => {
            tokio::spawn(interaction::handle(token, command, context));
        }
    }
}