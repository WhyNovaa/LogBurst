use crate::interfaces::grpc::LogCollectorService;
use crate::security::keystore::KeyStore;
use crate::server::Server;
use axum::Router;
use axum::extract::FromRef;
use config::grpc::GrpcConfig;
use config::http::HttpConfig;
use proto::log_proto::log_collector_server::LogCollectorServer;
use rdkafka::producer::FutureProducer;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

mod grpc;
mod middlewares;
mod v1;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub server: Arc<Server>,
}

pub fn router(server: Arc<Server>, key_store: Arc<KeyStore>) -> Router {
    let state = AppState { server };

    Router::new()
        .nest("/v1", v1::routes(key_store))
        .with_state(state)
}

pub async fn run_rest(
    server: Arc<Server>,
    cfg: HttpConfig,
    key_store: Arc<KeyStore>,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.url()).await?;

    let router = router(Arc::clone(&server), key_store);

    let shutdown_signal = server.token.clone();

    let rest = axum::serve(listener, router)
        .with_graceful_shutdown(async move { shutdown_signal.cancelled().await })
        .into_future();

    let deadline = async {
        server.token.cancelled().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
    };

    tokio::select! {
        v = rest => {
            info!("Rest finished");
            Ok(v?)
        }
        _ = deadline => {
            warn!("Rest server shutdown timeout");
            Ok(())
        }
    }
}

pub async fn run_grpc_server(
    key_store: Arc<KeyStore>,
    grpc_config: GrpcConfig,
    producer: FutureProducer,
    token: CancellationToken,
) -> Result<(), anyhow::Error> {
    let log_service = LogCollectorService {
        key_store,
        producer,
    };

    let addr = grpc_config
        .url()
        .parse()
        .expect("Invalid gRPC address format");
    tracing::info!("Starting gRPC LogCollector on {}", addr);

    Ok(tonic::transport::Server::builder()
        .add_service(LogCollectorServer::new(log_service))
        .serve_with_shutdown(addr, token.cancelled())
        .await?)
}
