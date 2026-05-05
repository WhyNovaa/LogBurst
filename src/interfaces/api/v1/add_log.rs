use crate::db::clickhouse::structs::Log;
use crate::interfaces::api::error::{ApiResult, IntoApiError};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::Value;

pub async fn add_log(
    State(sender): State<kanal::AsyncSender<Log>>,
    Json(payload): Json<Value>,
) -> ApiResult<impl IntoResponse> {
    let log = Log::try_from(payload).bad_request("Wrong log")?;

    Ok(sender.send(log).await.internal()?)
}
