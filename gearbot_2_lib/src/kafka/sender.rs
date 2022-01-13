use crate::kafka::base_kafka_config;
use bincode::config::Configuration;
use bincode::error::EncodeError;
use bincode::Encode;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use tracing::trace;

pub struct KafkaSender(FutureProducer);

#[allow(clippy::new_without_default)]
impl KafkaSender {
    pub async fn send<T>(&self, destination: &str, payload: &T) -> Result<(), KafkaSenderError>
    where
        T: Encode + Sized + Debug,
    {
        trace!("Sending message to {}: {:?}", destination, payload);
        let payload = bincode::encode_to_vec(payload, Configuration::standard())?;
        let _ = self
            .0
            .send(
                FutureRecord::<Vec<u8>, Vec<u8>>::to(destination).payload(&payload),
                Duration::from_secs(0),
            )
            .await?;

        Ok(())
    }

    // we make these on startup to panic is fine
    pub fn new() -> KafkaSender {
        KafkaSender(
            base_kafka_config()
                .set("compression.type", "gzip")
                .create()
                .expect("Failed to create kafka producer client"),
        )
    }
}

#[derive(Debug)]
pub enum KafkaSenderError {
    Encode(EncodeError),
    Kafka(KafkaError),
}

impl Error for KafkaSenderError {}

impl From<EncodeError> for KafkaSenderError {
    fn from(e: EncodeError) -> Self {
        KafkaSenderError::Encode(e)
    }
}

impl From<(KafkaError, OwnedMessage)> for KafkaSenderError {
    fn from(e: (KafkaError, OwnedMessage)) -> Self {
        let (e, _) = e;
        KafkaSenderError::Kafka(e)
    }
}

impl Display for KafkaSenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KafkaSenderError::Encode(e) => write!(f, "Failed to encode payload: {}", e),
            KafkaSenderError::Kafka(e) => write!(f, "Failed to send message to kafka: {}", e),
        }
    }
}
