use crate::config::grpc::GrpcConfig;
use crate::db::clickhouse::structs::Log;
use crate::server::Server;
use kanal::SendError;
use log_proto::{log_collector_server::{LogCollector, LogCollectorServer}, LogEntry, LogResponse};

use std::sync::Arc;
use time::OffsetDateTime;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

pub mod log_proto {
    tonic::include_proto!("log_collector");
}

pub struct LogCollectorService {
    server: Arc<Server>,
    sender: kanal::AsyncSender<Log>,
}

#[tonic::async_trait]
impl LogCollector for LogCollectorService {
    #[tracing::instrument(
        name = "grpc_ingestion",
        skip(self, request),
    )]
    async fn send_logs(
        &self,
        request: Request<Streaming<LogEntry>>,
    ) -> Result<Response<LogResponse>, Status> {
        tracing::info!("Starting reading logs...");

        let mut stream = request.into_inner();
        //let mut count_in_session = 0;
        let mut error_count = 0u32;
        while let Some(log_result) = stream.next().await {
            let log_entry: LogEntry = log_result.map_err(|e| {
                tracing::error!("Error while reading stream: {}", e);
                Status::internal("Failed to read log entry from stream")
            })?;

            let timestamp = if let Some(ts) = log_entry.timestamp {
                OffsetDateTime::from_unix_timestamp(ts.seconds)
                    .ok()
                    .map(|odt| odt.replace_nanosecond(ts.nanos as u32).unwrap_or(odt))
                    .unwrap_or_else(OffsetDateTime::now_utc) // Если конвертация не удалась
            } else {
                OffsetDateTime::now_utc()
            };

            if let Err(e) = self.sender.send(Log::from(log_entry)).await {
                tracing::error!(error = %e);
                error_count += 1;
            }

            // Обновляем общий счетчик
            /*count_in_session += 1;
            if let Ok(mut total) = self.state.logs_count.lock() {
                *total += 1;
            }*/
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

pub async fn run_grpc_server(
    server: Arc<Server>,
    grpc_config: GrpcConfig,
    log_sender: kanal::AsyncSender<Log>,
) -> Result<(), tonic::transport::Error> {
    let log_service = LogCollectorService {
        server,
        sender: log_sender,
    };

    let addr = grpc_config.url().parse().expect("Invalid gRPC address format");
    tracing::info!("Starting gRPC LogCollector on {}", addr);

    tonic::transport::Server::builder()
        .add_service(LogCollectorServer::new(log_service))
        .serve(addr)
        .await
}
