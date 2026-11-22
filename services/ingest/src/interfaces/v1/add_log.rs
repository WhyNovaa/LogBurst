use crate::server::Server;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use domain::log::Log;
use error::{ApiResult, IntoApiError};
use serde_json::Value;
use service_utils::kafka_log;
use std::sync::Arc;

pub async fn add_log(
    State(server): State<Arc<Server>>,
    Json(payload): Json<Value>,
) -> ApiResult<impl IntoResponse> {
    let log = Log::try_from(payload).bad_request("Wrong log")?;

    if let Err((e, _msg)) = kafka_log::send(&server.producer, log).await {
        tracing::error!("Failed to send KafkaRecord: {e}");
    }

    Ok(())
}
