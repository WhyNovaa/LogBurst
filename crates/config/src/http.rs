use dotenvy::dotenv;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: String,
    #[serde(deserialize_with = "string_to_bytes")]
    pub secret_key: Vec<u8>,
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

fn string_to_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.into_bytes())
}
