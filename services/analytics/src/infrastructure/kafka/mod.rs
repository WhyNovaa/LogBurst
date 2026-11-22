use config::kafka::KafkaConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use service_utils::kafka_log::{self, LOG_TOPIC};
use storage::clickhouse::ClickhouseLog;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub struct LiveLogs {
    consumer: StreamConsumer,
    sender: broadcast::Sender<ClickhouseLog>,
    token: CancellationToken,
}

impl LiveLogs {
    pub fn new(
        config: KafkaConfig,
        sender: broadcast::Sender<ClickhouseLog>,
        token: CancellationToken,
    ) -> anyhow::Result<Self> {
        // Each analytics instance needs all new logs, independently of the writer.
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers)
            .set(
                "group.id",
                format!("analytics-live-{:016x}", rand::random::<u64>()),
            )
            .set("auto.offset.reset", "latest")
            .set("enable.auto.commit", "false")
            .set("enable.auto.offset.store", "false")
            .create()?;
        consumer.subscribe(&[LOG_TOPIC])?;

        Ok(Self {
            consumer,
            sender,
            token,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = self.token.cancelled() => break,
                message = self.consumer.recv() => {
                    let message = match message {
                        Ok(message) => message,
                        Err(error) => {
                            tracing::error!("Failed to receive live log: {error}");
                            tokio::select! {
                                _ = self.token.cancelled() => break,
                                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {},
                            }
                            continue;
                        }
                    };
                    // Preserve the live stream's 1% sampling from the monolith.
                    if self.sender.receiver_count() == 0 || rand::random_range(1..=100) != 100 {
                        continue;
                    }
                    match kafka_log::decode(&message) {
                        Ok(log) => {
                            let _ = self.sender.send(ClickhouseLog::from_kafka(
                                log,
                                message.partition(),
                                message.offset(),
                            ));
                        }
                        Err(error) => tracing::error!("Invalid live log: {error:#}"),
                    }
                }
            }
        }
        Ok(())
    }
}
