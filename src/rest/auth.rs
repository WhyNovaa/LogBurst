use crate::db::pg::structs::User;
use crate::rest::error::{ApiError, ApiResult, IntoApiError};
use crate::rest::structs::{Claims, LoginRequest};
use crate::server::Server;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

pub async fn login(State(server): State<Arc<Server>>, Json(payload): Json<LoginRequest>) -> ApiResult<impl IntoResponse> {
    let is_authorized = server.auth_pool.login(User { username: payload.username.clone(), hashed_password: payload.hashed_password }).await?;

    if !is_authorized {
        return Err(ApiError::unauthorized("Wrong username or password"));
    }

    Ok(Claims::new(payload.username).internal()?)
}