use config::kafka::KafkaConfig;
use rdkafka::producer::FutureProducer;
use tokio_util::sync::CancellationToken;

pub struct Server {
    pub producer: FutureProducer,
    pub token: CancellationToken,
}

impl Server {
    pub fn new(config: KafkaConfig, token: CancellationToken) -> anyhow::Result<Self> {
        Ok(Self {
            producer: config.create_producer()?,
            token,
        })
    }
}
