use std::env;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::{Error, PgPool};
use thiserror::Error;
use crate::db::pg::crypto::hash;
use crate::models::http_client::api::handlers::auth::RegPayload;
use crate::traits::auth_repository::AuthRepository;
use crate::traits::new::AsyncNew;

pub struct AuthPool {
    pool: PgPool,
}

#[derive(Debug, Error)]
pub enum AuthRepositoryError {
    #[error("")]
    SqlxError(#[from] sqlx::Error)
}

impl IntoResponse for AuthRepositoryError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthRepositoryError::SqlxError(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "User not found")
            }
            AuthRepositoryError::SqlxError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl AsyncNew for AuthPool {
    async fn new() -> Self {
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER not found in .env file");
        let password =
            env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not found in .env file");
        let db = env::var("POSTGRES_DB").expect("POSTGRES_DB not found in .env file");

        let database_url = format!("postgres://{}:{}@pg:5432/{}", user, password, db);

        let pool = PgPool::connect(database_url.as_str())
            .await
            .expect("Connection error");

        Self { pool }
    }
}

impl AuthRepository for AuthPool {
    type Error = AuthRepositoryError;
    async fn create_user(&self, payload: RegPayload) -> Result<(), Self::Error> {
        let req = "INSERT INTO Users (login, hashed_password) VALUES ($1, $2)";

        let _ = sqlx::query(req)
            .bind(payload.login)
            .bind(hash(payload.password))
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

mod crypto {
    use ring::digest::{digest, SHA256};

    pub fn hash<T: AsRef<[u8]>>(data: T) -> String {
        let result = digest(&SHA256, data.as_ref());
        hex::encode(result)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn hash_test() {
            let data = b"test";
            let hash = hash(data);

            assert_eq!(hash, "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
        }
    }
}