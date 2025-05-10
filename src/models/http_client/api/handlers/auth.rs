use std::env;
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
use serde_json::json;
use crate::models::app::AuthCommandSender;

/// Key for encoding/decoding
static SECRET: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("SECRET").expect("SECRET not found in .env file")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
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

pub fn create_jwt(user_id: u32) -> anyhow::Result<String> {
    create_jwt_with_key(user_id, &SECRET)
}

pub fn validate_jwt(token: &str) -> anyhow::Result<Claims> {
    validate_jwt_with_key(token, &SECRET)
}

fn create_jwt_with_key(user_id: u32, key: &str) -> anyhow::Result<String> {
    log::info!("Creating jwt fot user_id {user_id}");

    let claims = Claims {
        sub: user_id.to_string(),
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
    State(_command_sender): State<AuthCommandSender>,
    Json(payload): Json<AuthPayload>
) -> Result<Json<AuthBody>, AuthError> {
    log::info!("Login endpoint {:?}", payload);

    if payload.login.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredentials)
    }

    if payload.login != "Admin" && payload.password != "Admin" {
        return Err(AuthError::WrongCredentials)
    }

    let jwt = create_jwt(1).unwrap(); //remove unwrap

    let body = AuthBody { jwt };

    Ok(Json(body))
}
pub async fn registration(
    State(_command_sender): State<AuthCommandSender>,
    payload: Json<RegPayload>,
)  {

    log::info!("Registration endpoint: {:?}", payload);
    // todo!
}

pub async fn protected(claims: Claims) -> Result<String, AuthError> {
    Ok(format!("Smth {:?}", claims))
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
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
        let key = "Test";

        let jwt = create_jwt_with_key(1, key).unwrap();

        let res = validate_jwt_with_key(&jwt, key);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn jwt_incorrect_validation() {
        let key = "Test";
        let invalid_key = "Invalid";

        let jwt = create_jwt_with_key(1, key).unwrap();

        let res = validate_jwt_with_key(&jwt, invalid_key);

        assert!(res.is_err());
    }
}