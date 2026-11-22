use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(sub: String, expiration_seconds: Duration) -> Self {
        let expiration = Utc::now() + expiration_seconds;

        Self {
            sub,
            exp: expiration.timestamp() as usize,
        }
    }
    pub fn into_jwt(&self, key: &[u8]) -> anyhow::Result<String> {
        Ok(jsonwebtoken::encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(key),
        )?)
    }

    pub fn from_token(token: &str, key: &[u8]) -> Option<Self> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(key),
            &Validation::default(),
        )
        .ok()
        .map(|d| d.claims)
    }
}
