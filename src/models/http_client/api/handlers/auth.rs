use std::env;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::{FromRequest, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{Json, RequestPartsExt};
use axum::response::{IntoResponse, Response};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::de::Unexpected::Str;
use serde_json::json;
use sqlx::FromRow;
use crate::models::app::AuthCommandSender;
use crate::models::http_client::api::handlers::auth_command::AuthCommand;

/// Key for encoding/decoding
static SECRET: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("SECRET").expect("SECRET not found in .env file")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    login: String,
    role: Role,
    exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct RegPayload {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    jwt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    User,
    Admin,
}
impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Role::User => "User",
            Role::Admin => "Admin",
        };

        write!(f, "{}", res)
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Admin" => Role::Admin,
            _ => Role::User,
        }
    }
}
impl Role {
    pub fn id(&self) -> i32 {
        match self {
            Role::User => 1,
            Role::Admin => 2,
        }
    }
}

pub fn create_jwt(login: String, role: Role) -> anyhow::Result<String> {
    create_jwt_with_key(login, role, &SECRET)
}

pub fn validate_jwt(token: &str) -> anyhow::Result<Claims> {
    validate_jwt_with_key(token, &SECRET)
}

fn create_jwt_with_key(login: String, role: Role, key: &str) -> anyhow::Result<String> {
    log::info!("Creating jwt fot user: {login}");

    let claims = Claims {
        login,
        role,
        exp: (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600) as usize,
    };
    let header = Header::new(Algorithm::HS256);

    Ok(encode(&header, &claims, &EncodingKey::from_secret(key.as_bytes()))?)
}

fn validate_jwt_with_key(token: &str, key: &str) -> anyhow::Result<Claims> {
    log::info!("Validating jwt: {token}");

    let decoding_key = DecodingKey::from_secret(key.as_bytes());
    let validation = Validation::default();

    Ok(decode::<Claims>(token, &decoding_key, &validation)?.claims)
}

pub async fn login(
    State(command_sender): State<AuthCommandSender>,
    Json(payload): Json<AuthPayload>
) -> Response {
    log::info!("Login endpoint {:?}", payload);

    if payload.login.is_empty() || payload.password.is_empty() {
        return AuthError::MissingCredentials.into_response()
    }

    let (one_s, one_r) = tokio::sync::oneshot::channel::<Response>();

    let command = AuthCommand::Login { login: payload.login, password: payload.password };

    send_command(&command_sender, command).await
}

pub async fn registration(
    State(command_sender): State<AuthCommandSender>,
    Json(payload): Json<RegPayload>,
) -> Response {
    log::info!("Registration endpoint: {:?}", payload);

    let command = AuthCommand::CreateUser {
        login: payload.login,
        password: payload.password,
        // default user's role is 'User'
        role: Role::User,
    };

    send_command(&command_sender, command).await
}

pub async fn protected(claims: Claims) -> Result<String, AuthError> {
    Ok(format!("Smth {:?}", claims))
}

async fn send_command(command_sender: &AuthCommandSender, command: AuthCommand) -> Response {
    let (one_s, one_r) = tokio::sync::oneshot::channel::<Response>();

    if let Err(e) = command_sender.send((command, one_s)).await {
        log::error!("Failed to send command: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal error"})),
        )
            .into_response();
    }

    one_r.await.unwrap_or_else(|e| {
        log::error!("oneshot receive failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal error"})),
        )
            .into_response()
    })
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InternalServerError,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let claims = validate_jwt(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(claims)
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn jwt_correct_validation() {
        let (login, role, key) = ("login".to_string(), Role::User, "key");

        let jwt = create_jwt_with_key(login, role, key).unwrap();

        let res = validate_jwt_with_key(&jwt, key);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn jwt_incorrect_validation() {
        let (login, role) = ("login".to_string(), Role::User);
        let (key, invalid_key) = ("key", "invalid_key");

        let jwt = create_jwt_with_key(login, role,  key).unwrap();

        let res = validate_jwt_with_key(&jwt, invalid_key);

        assert!(res.is_err());
    }
}