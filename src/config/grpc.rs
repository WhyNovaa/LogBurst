use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GrpcConfig {
    pub ip: String,
    pub port: String,
}

impl GrpcConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("GRPC_").from_env()
    }

    pub fn url(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}