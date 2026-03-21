use crate::config::rest::ServerConfig;
use crate::interfaces::api::error::{ApiError, ApiResult, IntoApiError};
use crate::interfaces::api::v1::dto::auth::{Claims, LoginRequest};
use crate::server::Server;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;

pub async fn login(
    State(server): State<Arc<Server>>,
    Extension(cfg): Extension<Arc<ServerConfig>>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let db_user = server
        .auth_pool
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

    Ok(Claims::new(payload.username)
        .into_jwt(&cfg.secret_key)
        .internal()?)
}
