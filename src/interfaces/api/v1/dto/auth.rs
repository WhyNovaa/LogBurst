use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    const TOKEN_EXPIRATION: Duration = Duration::from_secs(3600);

    pub fn new(sub: String) -> Self {
        let expiration = Utc::now() + Self::TOKEN_EXPIRATION;

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

#[cfg(test)]
mod tests {
    use crate::interfaces::api::v1::dto::auth::Claims;

    #[test]
    pub fn claims_encoding_decoding() {
        let key = vec![1, 2, 3];

        let claims = Claims::new("test".to_string());

        let encoded = claims.into_jwt(&key).unwrap();

        let decoded = Claims::from_token(encoded.as_str(), key.as_slice()).unwrap();

        assert_eq!(claims, decoded);
    }
}
