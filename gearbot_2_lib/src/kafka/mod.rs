use std::env;
use rdkafka::ClientConfig;

pub mod message;
pub mod sender;
pub mod listener;


pub fn base_kafka_config() -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", env::var("KAFKA_BOOTSTRAP").expect("Missing KAFKA_BOOTSTRAP env variable"));
    config
}




