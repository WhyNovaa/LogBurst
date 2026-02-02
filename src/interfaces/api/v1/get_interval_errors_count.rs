use crate::interfaces::api::error::{ApiResult, IntoApiError};
use crate::interfaces::api::v1::dto::get_interval_errors_count::IntervalQuery;
use crate::server::Server;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn get_interval_errors_count(
    State(server): State<Arc<Server>>,
    Query(query): Query<IntervalQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        server
            .logs_db
            .get_errors_count_interval(query.from, query.to)
            .await
            .internal()?,
    ))
}
