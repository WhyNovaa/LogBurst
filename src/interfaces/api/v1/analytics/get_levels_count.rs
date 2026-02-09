use crate::interfaces::api::error::{ApiResult, IntoApiError};
use crate::interfaces::api::v1::analytics::dto;
use crate::server::Server;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

pub async fn get_levels_count(
    State(server): State<Arc<Server>>,
) -> ApiResult<Json<dto::get_levels_count::LevelsCountBucket>> {
    let res = server.logs_db.get_levels_count().await.internal()?;

    Ok(Json(dto::get_levels_count::LevelsCountBucket::from(res)))
}
