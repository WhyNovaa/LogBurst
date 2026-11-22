use crate::interfaces::http::v1::dto;
use crate::server::Server;
use axum::Json;
use axum::extract::State;
use error::{ApiResult, IntoApiError};
use std::sync::Arc;

pub async fn get_levels_count(
    State(server): State<Arc<Server>>,
) -> ApiResult<Json<dto::get_levels_count::LevelsCountBucket>> {
    let res = server
        .analytics_queries
        .get_levels_count()
        .await
        .internal()?;

    Ok(Json(dto::get_levels_count::LevelsCountBucket::from(res)))
}
