use crate::interfaces::api::error::{ApiError, IntoApiError};
use crate::interfaces::api::v1::dto::auth::Claims;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;

#[derive(Clone)]
pub struct JwtState {
    pub secret_key: Vec<u8>,
}

pub async fn jwt_guard(
    State(state): State<JwtState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let jwt_opt = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match jwt_opt {
        None => return Err(ApiError::unauthorized("JWT not found")),
        Some(jwt) => {
            let jwt = jwt
                .strip_prefix("Bearer ")
                .unauthorized("Not Bearer auth")?;
            let claims = Claims::from_token(&jwt.trim(), &state.secret_key)
                .ok_or_else(|| ApiError::unauthorized("Invalid JWT"))?;

            if claims.exp <= Utc::now().timestamp() as usize {
                return Err(ApiError::unauthorized("JWT expired"));
            }
        }
    };

    Ok(next.run(req).await)
}
