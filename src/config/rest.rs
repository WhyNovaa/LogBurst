use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RestConfig {
    pub ip: String,
    pub port: String,
}

impl RestConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("SERVER_").from_env()
    }

    pub fn url(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}