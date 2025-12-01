use crate::db::clickhouse::structs::Log;
use crate::rest::error::{ApiResult, IntoApiError};
use crate::server::Server;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;
use std::sync::Arc;

pub async fn add_log(State(server): State<Arc<Server>>, State(sender): State<kanal::AsyncSender<Log>>, Json(payload): Json<Value>) -> ApiResult<impl IntoResponse> {
    let log = Log::from(payload);

    Ok(sender.send(log).await.internal()?)
}