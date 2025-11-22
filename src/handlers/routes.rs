use std::sync::Arc;
use axum::{middleware, Router};
use axum::routing::{get, post};
use crate::handlers::auth::{create_user, login, registration};
use crate::handlers::logs::{get_logs, save_log};
use crate::handlers::middlewares::check_admin_role;
use crate::models::app::{AuthCommandSender, LogCommandSender};
use crate::traits::logs_repository::LogsRepository;

pub fn auth_routes(auth_command_sender: AuthCommandSender) -> Router {
    Router::new()
        .nest("/auth", auth_protected_routes().merge(auth_public_routes()))
        .with_state(auth_command_sender)
}

pub fn logs_routes<L: LogsRepository>(logs_db: Arc<L>) -> Router {
    Router::new()
        .nest("/logs", logs_protected_routes().merge(logs_public_routes()))
        .with_state(logs_db)
}

// AUTH
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

// LOGS
fn logs_protected_routes<L: LogsRepository>() -> Router<Arc<L>> {
    Router::new()
        .route("/save", post(save_log))
        .route_layer(middleware::from_fn(check_admin_role))
}

fn logs_public_routes<L: LogsRepository>() -> Router<Arc<L>> {
    Router::new()
        .route("/", get(get_logs))
}