use std::sync::Arc;
use std::time::Duration;

use bincode::config::Configuration;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, StreamConsumer};
use rdkafka::groups::GroupInfo;
use rdkafka::util::Timeout;
use rdkafka::Message as AnnoyingConflict;
use tokio::sync::SetError;
use tracing::{debug, error, info, trace, warn};

use gearbot_2_lib::kafka::base_kafka_config;
use gearbot_2_lib::kafka::listener::new_listener;
use gearbot_2_lib::kafka::message::{General, Message};
use gearbot_2_lib::kafka::sender::{KafkaSender, KafkaSenderError};

use crate::util::bot_context::{BotContext, BotStatus};

mod general;
mod interaction;

pub async fn initialize_when_lonely(context: Arc<BotContext>) {
    if let Err(e) = KafkaSender::new()
        .send(&context.get_queue_topic(), &Message::General(General::Hello()))
        .await
    {
        error!("Failed to send hello message to the queue: {}", e);
    }
    debug!("Fetching group info");
    let consumer: BaseConsumer = base_kafka_config().create().unwrap();
    loop {
        {
            if context.is_status(BotStatus::Terminating) {
                return;
            }
            let metadata = consumer.fetch_group_list(
                Some(&context.get_queue_topic()),
                Timeout::After(Duration::from_secs(20)),
            );
            match metadata {
                Ok(metadata) => {
                    if let Some(info) = metadata
                        .groups()
                        .iter()
                        .filter(|group| *group.name() == context.get_queue_topic())
                        .collect::<Vec<&GroupInfo>>()
                        .first()
                    {
                        if info.members().is_empty() {
                            info!("No other instances listening on the queue, we are now the primary instance!");
                            context.set_status(BotStatus::Primary);
                            break;
                        }
                    } else {
                        info!("No consumer group exists so nobody can be handling the queue, proceeding");
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to get group metadata: {}", e);
                }
            }
        }
        if context.is_status(BotStatus::Primary) {
            //the regular mechanics kicked in and put us in charge of the queue, no need to keep trying anymore
            return;
        }
        trace!("Someone else is dealing with the queue, sleeping...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    if let Err(e) = initialize(context).await {
        error!("Failed to connect to the queue: {}", e);
    }
}

pub async fn initialize(context: Arc<BotContext>) -> Result<(), KafkaSenderError> {
    info!("Initializing kafka queue communication...");
    let topic = context.get_queue_topic();

    match new_listener(&[&topic], &topic) {
        // scope so this sender and listener can be dropped as soon as they are no longer needed
        Ok(listener) => {
            info!(
                "Communication link ready. Spawning background task to receive messages on topic '{}'",
                topic
            );
            let handle = tokio::spawn(receiver(listener, context.clone()));
            if let Err(e) = context.set_receiver_handle(handle) {
                error!("A receiver is already running! Aborting...");
                let handle = match e {
                    SetError::AlreadyInitializedError(handle) => handle,
                    SetError::InitializingError(handle) => handle,
                };
                handle.abort();
            } else if !context.is_status(BotStatus::Primary) {
                context.set_status(BotStatus::Primary);
            }
            Ok(())
        }
        Err(e) => Err(KafkaSenderError::Kafka(e)),
    }
}

async fn receiver(listener: StreamConsumer, context: Arc<BotContext>) {
    loop {
        let message = listener.recv().await;
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
                        }
                        Err(e) => {
                            error!(
                                "Failed to decode message on topic {}: {} ({:?})",
                                message.topic(),
                                e,
                                payload
                            )
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
        Message::General(message) => general::handle(message, context),
        Message::Interaction { token, command } => {
            if context.is_status(BotStatus::Primary) {
                tokio::spawn(interaction::handle(token, command, context));
            }
        }
    }
}
