use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::KafkaError;

use crate::config::WorkerSettings;

/// Build a consumer group client from env-derived settings.
pub fn stream_consumer(settings: &WorkerSettings) -> Result<StreamConsumer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", &settings.kafka_brokers)
        .set("group.id", &settings.kafka_group_id)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
}
