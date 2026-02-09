use crate::interfaces::api::error::{ApiResult, IntoApiError};
use crate::interfaces::api::v1::analytics::dto;
use crate::interfaces::api::v1::analytics::dto::get_levels_count_by_interval::LevelsCountIntervalBucket;
use crate::server::Server;
use axum::extract::{Query, State};
use axum::Json;
use std::sync::Arc;

pub async fn get_interval_levels_count(
    State(server): State<Arc<Server>>,
    Query(query): Query<dto::get_levels_count_by_interval::IntervalQuery>,
) -> ApiResult<Json<Vec<dto::get_levels_count_by_interval::LevelsCountIntervalBucket>>> {
    Ok(Json(
        server
            .logs_db
            .get_levels_count_by_interval(query.from, query.to)
            .await
            .internal()?
            .iter()
            .map(LevelsCountIntervalBucket::from_ref)
            .collect::<Vec<_>>(),
    ))
}
