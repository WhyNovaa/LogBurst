use crate::interfaces::http::v1::dto;
use crate::interfaces::http::v1::dto::get_levels_count_by_interval::LevelsCountIntervalBucket;
use crate::server::Server;
use axum::Json;
use axum::extract::{Query, State};
use error::{ApiResult, IntoApiError};
use std::sync::Arc;

pub async fn get_interval_levels_count(
    State(server): State<Arc<Server>>,
    Query(query): Query<dto::get_levels_count_by_interval::IntervalQuery>,
) -> ApiResult<Json<Vec<dto::get_levels_count_by_interval::LevelsCountIntervalBucket>>> {
    Ok(Json(
        server
            .analytics_queries
            .get_levels_count_by_interval(query.from, query.to)
            .await
            .internal()?
            .iter()
            .map(LevelsCountIntervalBucket::from_ref)
            .collect::<Vec<_>>(),
    ))
}
