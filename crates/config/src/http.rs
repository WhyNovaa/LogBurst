use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: String,
}

impl HttpConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("HTTP_").from_env()
    }

    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
