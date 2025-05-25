use axum::{middleware, Router};
use axum::routing::{get, post};
use crate::handlers::auth::{create_user, login, registration};
use crate::handlers::logs::{get_logs, save_log};
use crate::handlers::middlewares::check_admin_role;
use crate::models::app::{AuthCommandSender, LogCommandSender};

pub fn auth_routes(auth_command_sender: AuthCommandSender) -> Router {
    Router::new()
        .nest("/auth", auth_protected_routes().merge(auth_public_routes()))
        .with_state(auth_command_sender)
}

pub fn logs_routes(log_command_sender: LogCommandSender) -> Router {
    Router::new()
        .nest("/logs", logs_protected_routes().merge(logs_public_routes()))
        .with_state(log_command_sender)
}

fn auth_protected_routes() -> Router<AuthCommandSender> {
    Router::new()
        .route("/create", post(create_user))
        .route_layer(middleware::from_fn(check_admin_role))
}

fn auth_public_routes() -> Router<AuthCommandSender> {
    Router::new()
        .route("/reg", post(registration))
        .route("/login", post(login))
}

fn logs_protected_routes() -> Router<LogCommandSender> {
    Router::new()
        .route("/save", post(save_log))
        .route_layer(middleware::from_fn(check_admin_role))
}

fn logs_public_routes() -> Router<LogCommandSender> {
    Router::new()
        .route("/", get(get_logs))
}