use crate::LOG_GROUP_ID;
use config::clickhouse::ClickhouseConfig;
use config::kafka::KafkaConfig;
use rdkafka::consumer::StreamConsumer;
use tokio_util::sync::CancellationToken;

pub struct Writer {
    pub consumer: StreamConsumer,
    pub client: clickhouse::Client,
    pub token: CancellationToken,
}

impl Writer {
    pub async fn new(
        kafka_cfg: KafkaConfig,
        clickhouse_cfg: ClickhouseConfig,
        token: CancellationToken,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            consumer: kafka_cfg.create_consumer(LOG_GROUP_ID)?,
            client: clickhouse_cfg.client(),
            token,
        })
    }

    pub async fn start_receiving(self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                msg = self.consumer.recv() => {
                    match msg {

                    }
                },
                _ = self.token.cancelled() => {

                }
            }
        }
    }
}
