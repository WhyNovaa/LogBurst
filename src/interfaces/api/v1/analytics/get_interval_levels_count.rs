use crate::interfaces::api::error::{ApiResult, IntoApiError};
use crate::interfaces::api::v1::analytics::dto::get_interval_levels_count::{
    ErrorBucket, IntervalQuery,
};
use crate::server::Server;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn get_interval_levels_count(
    State(server): State<Arc<Server>>,
    Query(query): Query<IntervalQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        server
            .logs_db
            .get_levels_count_by_interval(query.from, query.to)
            .await
            .internal()?
            .iter()
            .map(ErrorBucket::from_ref)
            .collect::<Vec<_>>(),
    ))
}
