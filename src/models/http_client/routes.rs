use axum::handler::Handler;
use axum::Router;
use axum::routing::{get, post};
use crate::models::app::{AuthCommandSender, LogSender};
use crate::models::http_client::api::handlers::auth::{login, protected, registration};

pub fn auth_routes(auth_command_sender: AuthCommandSender) -> Router {
    Router::new()
        .route("/reg", post(registration))
        .route("/login", post(login))
        .route("/protected", get(protected))
        .with_state(auth_command_sender)
}

