use crate::interfaces::api::v1::dto::auth::Claims;
use axum::body::Body;
use axum::http::header::AUTHORIZATION;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;

pub async fn jwt_guard(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let jwt_opt = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match jwt_opt {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(jwt) => {
            let claims = Claims::from_token(&jwt.trim()).ok_or_else(|| StatusCode::UNAUTHORIZED)?;

            if claims.exp <= Utc::now().timestamp() as usize {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    };

    Ok(next.run(req).await)
}
