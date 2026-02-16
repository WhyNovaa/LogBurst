use crate::config::rest::RestConfig;
use crate::db::clickhouse::structs::Log;
use crate::security::keystore::KeyStore;
use crate::server::Server;
use axum::Router;
use axum::extract::FromRef;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

mod error;
mod middlewares;
pub mod v1;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub server: Arc<Server>,
    pub log_sender: kanal::AsyncSender<Log>,
    pub live_log_sender: tokio::sync::broadcast::Sender<Log>,
}

pub fn routes(
    server: Arc<Server>,
    log_sender: kanal::AsyncSender<Log>,
    live_log_sender: tokio::sync::broadcast::Sender<Log>,
    key_store: Arc<KeyStore>,
) -> Router {
    Router::new()
        .nest("/v1", v1::routes(key_store))
        .with_state(AppState {
            server,
            log_sender,
            live_log_sender,
        })
}

pub async fn run_rest(
    server: Arc<Server>,
    cfg: RestConfig,
    log_sender: kanal::AsyncSender<Log>,
    live_log_sender: tokio::sync::broadcast::Sender<Log>,
    key_store: Arc<KeyStore>,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.url()).await?;

    let router = routes(Arc::clone(&server), log_sender, live_log_sender, key_store);

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
