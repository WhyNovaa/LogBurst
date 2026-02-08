use crate::interfaces::api::error::{ApiError, ApiResult, IntoApiError};
use crate::interfaces::api::v1::dto::auth::{Claims, LoginRequest};
use crate::server::Server;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, LazyLock};

pub static SECRET_KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    dotenv().ok();
    env::var("SECRET_KEY")
        .expect("SECRET_KEY must be set in .env")
        .into_bytes()
});

pub async fn login(
    State(server): State<Arc<Server>>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let db_user = server
        .auth_pool
        .get_user_by_username(&payload.username)
        .await?
        .ok_or_else(|| ApiError::unauthorized("Wrong username or password"))?;

    let parsed_hash = PasswordHash::new(&db_user.hashed_password).internal()?;

    let verified = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !verified {
        return Err(ApiError::unauthorized("Wrong username or password"));
    }

    Ok(Claims::new(payload.username)
        .into_jwt(&*SECRET_KEY)
        .internal()?)
}
