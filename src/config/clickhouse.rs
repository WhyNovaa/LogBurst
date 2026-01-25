use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ClickhouseConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
}

impl ClickhouseConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("CLICKHOUSE_").from_env()
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
