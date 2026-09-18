use crate::LOG_TOPIC;
use crate::security::keystore::KeyStore;
use domain::log::Log;
use proto::log_proto::log_collector_server::LogCollector;
use proto::log_proto::{LogResponse, SignedLogEntry};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::sync::Arc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

const LOG_SECS_TO_PROCEED: i64 = 120;

pub struct LogCollectorService {
    pub(crate) key_store: Arc<KeyStore>,
    pub(crate) producer: FutureProducer,
}

#[tonic::async_trait]
impl LogCollector for LogCollectorService {
    #[tracing::instrument(name = "grpc_ingestion", skip(self, request))]
    async fn send_logs(
        &self,
        request: Request<Streaming<SignedLogEntry>>,
    ) -> Result<Response<LogResponse>, Status> {
        tracing::info!("Starting reading logs...");

        let mut stream = request.into_inner();

        let mut error_count = 0u32;
        while let Some(result) = stream.next().await {
            let signed_log = match result {
                Ok(signed) => signed,
                Err(e) => {
                    tracing::error!("Error while reading stream: {}", e);
                    return Err(Status::internal("Failed to read log entry from stream"));
                }
            };

            let log = verify_signed_log(&self.key_store, signed_log).await?;

            let serialized = postcard::to_allocvec(&log)
                .map_err(|_| Status::internal("Failed to serialize log"))?;

            let record: FutureRecord<(), _> =
                FutureRecord::to(LOG_TOPIC).payload(serialized.as_slice());

            if let Err((e, _msg)) = self
                .producer
                .send(record, std::time::Duration::from_secs(1))
                .await
            {
                tracing::error!("Failed to send KafkaRecord: {e}");
                error_count += 1;
            }
        }

        tracing::info!("Session ended. Errors: {error_count}");

        Ok(Response::new(LogResponse {
            success: true,
            error_message: {
                if error_count > 0 {
                    format!("{error_count} occurred")
                } else {
                    "".to_string()
                }
            },
        }))
    }
}

pub async fn verify_signed_log(
    key_store: &KeyStore,
    signed: SignedLogEntry,
) -> Result<Log, tonic::Status> {
    let service_id = signed.service_id;
    let log_entry = signed
        .log
        .ok_or_else(|| Status::invalid_argument("missing log"))?;
    let timestamp = log_entry
        .timestamp
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("missing timestamp in log"))?;

    if timestamp.seconds > chrono::Utc::now().timestamp() + LOG_SECS_TO_PROCEED {
        return Err(Status::unauthenticated("message is too old"));
    }

    let key = key_store
        .get(&service_id)
        .ok_or_else(|| Status::not_found("service not found"))?;

    let payload = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        signed.signed_at,
        service_id,
        timestamp.seconds,
        log_entry.level,
        log_entry.service,
        log_entry.message
    );

    let mut hasher = blake3::Hasher::new_keyed(&key);

    let signed_payload = hasher.update(payload.as_bytes()).finalize().to_hex();

    if !const_time_compare_hashes(signed_payload.as_bytes(), signed.signature.as_bytes()) {
        return Err(tonic::Status::unauthenticated("invalid signature"));
    }

    Log::try_from(log_entry)
}

fn const_time_compare_hashes(a: &[u8], b: &[u8]) -> bool {
    let is_valid = subtle::ConstantTimeEq::ct_eq(a, b);

    bool::from(is_valid)
}
