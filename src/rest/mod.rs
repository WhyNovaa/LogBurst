mod auth;
mod structs;
mod error;

use crate::config::Config;
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

async fn run_rest(server: Arc<Server>, cfg: Arc<Config>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.rest_cfg.url())
        .await
        .unwrap();

    let router = get_router(Arc::clone(&server));

    let shutdown_signal = server.token.clone();

    let rest = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown_signal.cancelled().await
        })
        .into_future();

    let deadline = async {
        server.token.cancelled().await; // Тут можно оставить как есть, т.к. это локальный await
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
pub fn get_router(server: Arc<Server>) -> Router {
    Router::new()
        .route("/login", post(auth::login))
        .with_state(server)
}