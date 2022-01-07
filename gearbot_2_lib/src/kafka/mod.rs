use std::env;
use rdkafka::ClientConfig;
use rdkafka::config::RDKafkaLogLevel;

pub mod message;
pub mod sender;
pub mod listener;


fn base_kafka_config() -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", env::var("KAFKA_BOOTSTRAP").expect("Missing KAFKA_BOOTSTRAP env variable"))
        .set_log_level(RDKafkaLogLevel::Debug);
    config
}




