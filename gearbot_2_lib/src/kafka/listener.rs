use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use crate::kafka::base_kafka_config;

pub fn new_listener(topics: &[&str], group_id: Option<&str>) -> Result<StreamConsumer, KafkaError> {
    let mut config = base_kafka_config();
    if let Some(group_id) = group_id {
        config.set("group.id", group_id);
    }
    let consumer: StreamConsumer = config.set("allow.auto.create.topics", "true")
        .create()?;

    consumer.subscribe(topics)?;

    Ok(consumer)
}