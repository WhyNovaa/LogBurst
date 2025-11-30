use crate::rest::SECRET_KEY;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TOKEN_EXPIRATION: Duration = Duration::from_secs(3600);

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub hashed_password: String,
}

#[derive(Serialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}


impl Claims {
    pub fn new(sub: String) -> anyhow::Result<String> {
        let expiration = Utc::now() + TOKEN_EXPIRATION;

        let claims = Claims {
            sub,
            exp: expiration.timestamp() as usize,
        };

        Ok(jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(&*SECRET_KEY))?)
    }
}