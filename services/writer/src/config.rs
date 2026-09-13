use config::clickhouse::ClickhouseConfig;
use config::kafka::KafkaConfig;

pub struct Config {
    pub kafka: KafkaConfig,
    pub clickhouse: ClickhouseConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            kafka: KafkaConfig::from_env()?,
            clickhouse: ClickhouseConfig::from_env()?,
        })
    }
}
