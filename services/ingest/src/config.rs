use config::grpc::GrpcConfig;
use config::http::HttpConfig;
use config::kafka::KafkaConfig;
use config::postgres::PostgresConfig;

pub struct Config {
    pub http: HttpConfig,
    pub grpc: GrpcConfig,
    pub kafka: KafkaConfig,
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            http: HttpConfig::from_env()?,
            grpc: GrpcConfig::from_env()?,
            kafka: KafkaConfig::from_env()?,
            postgres: PostgresConfig::from_env()?,
        })
    }
}
