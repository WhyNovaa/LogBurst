use config::grpc::GrpcConfig;
use config::http::HttpConfig;
use config::kafka::KafkaConfig;

pub struct Config {
    pub http: HttpConfig,
    pub grpc: GrpcConfig,
    pub kafka: KafkaConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            http: HttpConfig::from_env()?,
            grpc: GrpcConfig::from_env()?,
            kafka: KafkaConfig::from_env()?,
        })
    }
}
