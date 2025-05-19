use axum::Router;
use axum::routing::{get, post};
use crate::handlers::auth::{create_user, login, registration};
use crate::handlers::logs::{get_logs, save_log};
use crate::models::app::{AuthCommandSender, LogCommandSender};

pub fn auth_routes(auth_command_sender: AuthCommandSender) -> Router {
    Router::new()
        .nest("/auth",
            Router::new()
                .route("/reg", post(registration))
                .route("/login", post(login))
                .route("/create", post(create_user))
                .with_state(auth_command_sender)
        )
}

pub fn logs_routes(log_command_sender: LogCommandSender) -> Router {
    Router::new()
        .nest("/logs",
            Router::new()
                .route("/save", post(save_log))
                .route("/get/{service}", get(get_logs))
                .with_state(log_command_sender)
        )
}
