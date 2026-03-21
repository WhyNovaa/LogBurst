use crate::interfaces::api::v1::analytics::get_levels_count::get_levels_count;
use crate::interfaces::api::v1::analytics::get_levels_count_by_interval::get_interval_levels_count;
use crate::interfaces::api::v1::analytics::get_logs_by_interval::get_logs_by_interval;
use crate::interfaces::api::v1::analytics::get_services::get_services;
use crate::interfaces::api::v1::analytics::stream_last_logs::stream_last_logs;
use crate::interfaces::api::AppState;
use axum::routing::get;
use axum::Router;

mod dto;
mod get_levels_count;
mod get_levels_count_by_interval;
mod get_logs_by_interval;
mod get_services;
mod stream_last_logs;

pub fn routes() -> Router<AppState> {
    let analytics = Router::new()
        .route("/levels/intervals", get(get_interval_levels_count))
        .route("/levels/count", get(get_levels_count))
        .route("/last", get(stream_last_logs))
        .route("/logs/interval", get(get_logs_by_interval))
        .route("/services", get(get_services));

    Router::new().nest("/analytics", analytics)
}
