use crate::db::clickhouse::structs::Log;
use crate::rest::error::{ApiResult, IntoApiError};
use crate::server::Server;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::Value;
use std::sync::Arc;

pub async fn add_log(
    State(server): State<Arc<Server>>,
    State(sender): State<kanal::AsyncSender<Log>>,
    Json(payload): Json<Value>,
) -> ApiResult<impl IntoResponse> {
    let log = Log::try_from(payload).internal()?;

    Ok(sender.send(log).await.internal()?)
}
