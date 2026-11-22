use crate::interfaces::http::v1::dto::login::LoginRequest;
use crate::security::jwt::Claims;
use crate::server::Server;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use config::jwt::JwtConfig;
use error::{ApiError, ApiResult, IntoApiError};
use std::sync::Arc;

pub async fn login(
    State(server): State<Arc<Server>>,
    Extension(cfg): Extension<Arc<JwtConfig>>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let db_user = server
        .user_queries
        .get_user_by_username(&payload.username)
        .await
        .internal()?
        .ok_or_else(|| ApiError::unauthorized("Wrong username or password"))?;

    let parsed_hash = PasswordHash::new(&db_user.hashed_password).internal()?;

    let verified = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !verified {
        return Err(ApiError::unauthorized("Wrong username or password"));
    }

    Claims::new(payload.username, cfg.expiration_seconds)
        .into_jwt(&cfg.secret_key)
        .internal()
}
