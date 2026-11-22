use dotenvy::dotenv;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

#[derive(Deserialize)]
pub struct JwtConfig {
    #[serde(deserialize_with = "string_to_bytes")]
    pub secret_key: Vec<u8>,
    #[serde(deserialize_with = "u64_to_duration")]
    pub expiration_seconds: Duration,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("JWT_").from_env()
    }
}

fn string_to_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.into_bytes())
}

fn u64_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}
