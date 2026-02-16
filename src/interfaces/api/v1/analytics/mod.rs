use crate::interfaces::api::AppState;
use crate::interfaces::api::v1::analytics::get_levels_count::get_levels_count;
use crate::interfaces::api::v1::analytics::get_levels_count_by_interval::get_interval_levels_count;
use crate::interfaces::api::v1::analytics::get_logs_by_interval::get_logs_by_interval;
use crate::interfaces::api::v1::analytics::stream_last_logs::stream_last_logs;
use axum::Router;
use axum::routing::get;

mod dto;
mod get_levels_count;
mod get_levels_count_by_interval;
mod get_logs_by_interval;
mod stream_last_logs;

pub fn routes() -> Router<AppState> {
    let analytics = Router::new()
        .route("/levels/intervals", get(get_interval_levels_count))
        .route("/levels/count", get(get_levels_count))
        .route("/last", get(stream_last_logs))
        .route("/logs/interval", get(get_logs_by_interval));

    Router::new().nest("/analytics", analytics)
}
