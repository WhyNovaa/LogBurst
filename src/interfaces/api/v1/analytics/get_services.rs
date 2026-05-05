use crate::interfaces::api::error::{ApiResult, IntoApiError};
use crate::server::Server;
use axum::Json;
use axum::extract::State;
use std::sync::Arc;

pub async fn get_services(State(server): State<Arc<Server>>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(server.logs_db.get_services().await.internal()?))
}
