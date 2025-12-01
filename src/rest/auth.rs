use crate::rest::error::{ApiError, ApiResult, IntoApiError};
use crate::rest::structs::{Claims, LoginRequest};
use crate::server::Server;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

pub async fn login(State(server): State<Arc<Server>>, Json(payload): Json<LoginRequest>) -> ApiResult<impl IntoResponse> {
    let db_user = server.auth_pool.get_user_by_username(&payload.username).await?.ok_or_else(|| ApiError::unauthorized("Wrong username or password"))?;

    let parsed_hash =
        PasswordHash::new(&db_user.hashed_password).internal()?;

    let verified = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !verified {
        return Err(ApiError::unauthorized("Wrong username or password"));
    }

    Ok(Claims::new(payload.username).internal()?)
}
