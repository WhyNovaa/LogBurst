use crate::db::clickhouse::structs::Log;
use crate::rest::SECRET_KEY;
use crate::server::Server;
use axum::extract::FromRef;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
const TOKEN_EXPIRATION: Duration = Duration::from_secs(3600);

#[derive(Clone, FromRef)]
pub struct AppState {
    pub server: Arc<Server>,
    pub log_sender: kanal::AsyncSender<Log>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
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