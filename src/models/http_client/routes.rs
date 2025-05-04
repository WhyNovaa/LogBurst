use axum::Router;
use axum::routing::post;
use crate::models::http_client::api::handlers::auth::{login, registration};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/reg", post(registration))
        .route("/login", post(login))
}

