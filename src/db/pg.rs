use std::env;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::{Error, PgPool};
use sqlx::error::ErrorKind::UniqueViolation;
use thiserror::Error;
use tokio::sync::oneshot::Sender;
use crate::db::pg::crypto::hash;
use crate::models::app::AuthCommandReceiver;
use crate::models::http_client::api::handlers::auth::{create_jwt, AuthError, RegPayload, Role};
use crate::models::http_client::api::handlers::auth_command::AuthCommand;
use crate::models::user::{DbUser, User};
use crate::traits::auth_repository::AuthRepository;
use crate::traits::start::Start;

pub struct AuthPool {
    pool: PgPool,
    auth_command_receiver: AuthCommandReceiver,
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
            },
            AuthRepositoryError::SqlxError(sqlx::Error::Database(e)) if e.kind() == UniqueViolation => {
                (StatusCode::BAD_REQUEST, "User with this login already exists")
            },
            AuthRepositoryError::SqlxError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

#[async_trait]
impl Start for AuthPool {
    async fn start(mut self) {
        loop {
           if let Some((command, response_sender)) = self.auth_command_receiver.recv().await {
               log::info!("Auth: {:?}", command);

               let response = self.handle_command(command).await;

               response_sender.send(response).unwrap();
           }
        }
    }
}

impl AuthRepository for AuthPool {
    type Error = AuthRepositoryError;

    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self {
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER not found in .env file");
        let password =
            env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not found in .env file");
        let db = env::var("POSTGRES_DB").expect("POSTGRES_DB not found in .env file");

        let database_url = format!("postgres://{}:{}@pg:5432/{}", user, password, db);

        let pool = PgPool::connect(database_url.as_str())
            .await
            .expect("Connection error");

        Self {
            pool,
            auth_command_receiver,
        }
    }

    async fn create_user(&self, login: &str, password: &str, role_id: i32) -> Result<(), Self::Error> {
        let req = "INSERT INTO users (login, hashed_password, role_id) VALUES ($1, $2, $3)";

        let _ = sqlx::query(req)
            .bind(login)
            .bind(hash(password))
            .bind(role_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_user_by_login(&self, login: &str) -> Result<Option<User>, Self::Error> {
        let req = "SELECT u.login, u.hashed_password, r.name AS role_name FROM users u INNER JOIN roles r ON u.role_id = r.id WHERE u.login = $1";

        let maybe_db_user = sqlx::query_as::<_, DbUser>(req)
            .bind(login)
            .fetch_optional(&self.pool)
            .await?;

        println!("{maybe_db_user:?}");

        let maybe_user = maybe_db_user.map(|u| User {
            login: u.login,
            hashed_password: u.hashed_password,
            role: Role::from(u.role_name),
        });

        Ok(maybe_user)
    }
}

impl AuthPool {
    pub async fn handle_command(&self, command: AuthCommand) -> Response {
        match command {
            AuthCommand::CreateUser {login, password, role}=> {
                match self.create_user(&login, &password, role.id()).await {
                    Ok(_) => {
                        let body = Json(json!({
                            "message": "User created successfully"
                        }));

                        (StatusCode::CREATED, body).into_response()
                    }
                    Err(e) => e.into_response(),
                }
            }
            AuthCommand::Login {login, password} => {
                match self.get_user_by_login(&login).await {
                    Ok(Some(user)) if user.hashed_password == hash(password) => {
                        match create_jwt(login.clone(), user.role) {
                            Ok(jwt) => {
                                log::info!("User: {login} authorized successfully");

                                let body = Json(json!({ "jwt": jwt }));
                                (StatusCode::OK, body).into_response()
                            }
                            Err(_) => {
                                log::error!("Error while creating jwt");

                                AuthError::InternalServerError.into_response()
                            },
                        }
                    }
                    Ok(Some(_)) | Ok(None) => {
                        log::info!("User: {login}, wrong credentials");
                        AuthError::WrongCredentials.into_response()
                    }
                    Err(e) => {
                        log::error!("AuthRepositoryError: {e}");
                        e.into_response()
                    },
                }
            }
        }
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