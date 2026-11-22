mod get_levels_count;
mod get_levels_count_by_interval;
mod get_logs_by_interval;
mod get_services;
mod login;
mod stream_last_logs;

use crate::server::Server;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn analytics_routes() -> Router<Arc<Server>> {
    Router::new()
        .route(
            "/levels/intervals",
            get(get_levels_count_by_interval::get_interval_levels_count),
        )
        .route("/levels/count", get(get_levels_count::get_levels_count))
        .route("/last", get(stream_last_logs::stream_last_logs))
        .route(
            "/logs/interval",
            get(get_logs_by_interval::get_logs_by_interval),
        )
        .route("/services", get(get_services::get_services))
}

pub fn auth_routes() -> Router<Arc<Server>> {
    Router::new().route("/auth", post(login::login))
}
