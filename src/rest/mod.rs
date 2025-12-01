mod auth;
mod structs;
mod error;
mod logs;

use crate::config::Config;
use crate::db::clickhouse::structs::Log;
use crate::rest::structs::AppState;
use crate::server::Server;
use axum::routing::post;
use axum::Router;
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tracing::log::info;
use tracing::warn;

pub static SECRET_KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    dotenv().ok();
    env::var("SECRET_KEY")
        .expect("SECRET_KEY must be set in .env")
        .into_bytes()
});

pub async fn run_rest(server: Arc<Server>, cfg: Arc<Config>, log_sender: kanal::AsyncSender<Log>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.rest_cfg.url())
        .await
        .unwrap();

    let router = get_router(Arc::clone(&server), log_sender);

    let shutdown_signal = server.token.clone();

    let rest = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown_signal.cancelled().await
        })
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
pub fn get_router(server: Arc<Server>, log_sender: kanal::AsyncSender<Log>) -> Router {
    let auth_router = Router::new()
        .route("/auth/login", post(auth::login));

    let logs_router = Router::new()
        .route("/logs", post(logs::add_log));

    auth_router.merge(logs_router).with_state(AppState {
        server,
        log_sender,
    })
}