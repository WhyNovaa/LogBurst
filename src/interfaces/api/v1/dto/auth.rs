use crate::interfaces::api::v1::auth::SECRET_KEY;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    const TOKEN_EXPIRATION: Duration = Duration::from_secs(3600);

    pub fn new(sub: String) -> anyhow::Result<String> {
        let expiration = Utc::now() + Self::TOKEN_EXPIRATION;

        let claims = Claims {
            sub,
            exp: expiration.timestamp() as usize,
        };

        Ok(jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&*SECRET_KEY),
        )?)
    }

    pub fn from_token(token: &str) -> Option<Self> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(&*SECRET_KEY),
            &Validation::default(),
        )
        .ok()
        .map(|d| d.claims)
    }
}
