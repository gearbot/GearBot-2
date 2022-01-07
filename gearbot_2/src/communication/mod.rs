use std::sync::Arc;
use bincode::config::Configuration;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::Message as AnnoyingConflict;
use tracing::{error, info, trace, warn};
use gearbot_2_lib::kafka::listener::new_listener;
use gearbot_2_lib::kafka::message::{InteractionCommand, Message};
use crate::BotContext;
use crate::util::BotStatus;

mod general;
mod interaction;


// we only need one so fine to crash out on startup
pub fn initialize(context: Arc<BotContext>) -> Result<(), KafkaError> {
    info!("Establishing kafka communication link...");
    let topic = format!("{}_cluster_{}", context.cluster_info.cluster_identifier, context.cluster_info.cluster_id);
    match new_listener(&[&topic], Some(&context.cluster_info.cluster_identifier)) {
        Ok(listener) => {
            info!("Communication link ready. Spawning background task to receive messages on topic '{}'", topic);
            tokio::spawn(receiver(listener, context));
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}

async fn receiver(listener: StreamConsumer, context: Arc<BotContext>) {
    loop {
        match listener.recv().await {
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

                            // Check if the cluster should remain up
                            // we only check after we handled the message so a message can be used to put this server into shutting down mode
                            let status = context.status();
                            match &status {
                                BotStatus::STARTING | BotStatus::STANDBY => {
                                    warn!("Got a cluster command while in {} mode, meaning nobody else was listening to the queue. Moving into primary cluster mode!", status.name())
                                }
                                BotStatus::PRIMARY => {}
                                BotStatus::TERMINATING => {
                                    info!("Cluster shutting down, terminating kafka message receiver");
                                    return;
                                }
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
        Message::General(message) => {}
        Message::Interaction { token, command } => {
            tokio::spawn(interaction::handle(token, command, context));
        }
    }
}