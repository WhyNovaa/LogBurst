use crate::server::Server;
use crate::LOG_TOPIC;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use domain::log::Log;
use error::{ApiResult, IntoApiError};
use rdkafka::producer::FutureRecord;
use serde_json::Value;
use std::sync::Arc;

pub async fn add_log(
    State(server): State<Arc<Server>>,
    Json(payload): Json<Value>,
) -> ApiResult<impl IntoResponse> {
    let log = Log::try_from(payload).bad_request("Wrong log")?;

    // todo replace with smth else
    let serialized = postcard::to_allocvec(&log).internal()?;

    let record: FutureRecord<(), _> = FutureRecord::to(LOG_TOPIC).payload(serialized.as_slice());

    if let Err((e, _msg)) = server
        .producer
        .send(record, std::time::Duration::from_secs(1))
        .await
    {
        tracing::error!("Failed to send KafkaRecord: {e}");
    }

    Ok(())
}
