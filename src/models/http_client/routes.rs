use axum::Router;
use axum::routing::{get, post};
use crate::models::http_client::api::handlers::auth::{login, protected, registration};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/reg", post(registration))
        .route("/login", post(login))
        .route("/protected", get(protected))
}

