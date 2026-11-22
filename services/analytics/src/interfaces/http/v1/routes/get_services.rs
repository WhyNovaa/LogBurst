use crate::server::Server;
use axum::Json;
use axum::extract::State;
use error::{ApiResult, IntoApiError};
use std::sync::Arc;

pub async fn get_services(State(server): State<Arc<Server>>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(
        server.analytics_queries.get_services().await.internal()?,
    ))
}
