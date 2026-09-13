use dotenvy::dotenv;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub partitions_number: u8,
}

impl KafkaConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("KAFKA_").from_env()
    }

    pub fn create_producer(&self) -> anyhow::Result<FutureProducer> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", self.brokers.as_str())
            .set("enable.idempotence", "true")
            .set("linger.ms", "5")
            .set("compression.type", "lz4")
            .set("message.timeout.ms", "10000")
            .create()?;

        Ok(producer)
    }

    pub fn create_consumer(&self, group_id: &str) -> anyhow::Result<StreamConsumer> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", self.brokers.as_str())
            .set("enable.partition.eof", "false")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.offset.store", "false")
            .create()?;

        Ok(consumer)
    }
}
