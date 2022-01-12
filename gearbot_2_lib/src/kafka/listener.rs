use crate::kafka::base_kafka_config;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;

pub fn new_listener(topics: &[&str], group_id: &str) -> Result<StreamConsumer, KafkaError> {
    let mut config = base_kafka_config();
    config.set("client.id", "gearbot");
    config.set("group.id", group_id);
    let consumer: StreamConsumer = config.set("allow.auto.create.topics", "true").create()?;

    consumer.subscribe(topics)?;

    Ok(consumer)
}
