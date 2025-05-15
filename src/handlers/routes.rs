use axum::handler::Handler;
use axum::Router;
use axum::routing::post;
use crate::handlers::auth::{create_user, login, registration};
use crate::models::app::AuthCommandSender;

pub fn auth_routes(auth_command_sender: AuthCommandSender) -> Router {
    Router::new()
        .nest("/auth",
            Router::new()
                .route("/reg", post(registration))
                .route("/login", post(login))
                .route("/create", post(create_user))
                .with_state(auth_command_sender))
}

