use crate::interfaces::{run_grpc_server, run_rest};
use crate::security::keystore::KeyStore;
use crate::security::update_key_store;
use crate::server::Server;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod interfaces;
mod security;
mod server;

pub const LOG_TOPIC: &str = "logs";

pub fn init_fmt() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info").add_directive("clickhouse=debug".parse().unwrap())
    });

    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_fmt();

    let config = config::Config::from_env()?;

    let token = CancellationToken::new();

    let server = Arc::new(Server::new(config.kafka, token.clone())?);

    let key_store = Arc::new(KeyStore::new(KeyStore::load()?));

    let mut supervisor: JoinSet<anyhow::Result<()>> = JoinSet::new();

    supervisor.spawn(update_key_store(server.token.clone(), key_store.clone()));

    // Rest
    supervisor.spawn(run_rest(
        Arc::clone(&server),
        config.http,
        Arc::clone(&key_store),
    ));

    // gRPC
    supervisor.spawn(run_grpc_server(
        Arc::clone(&key_store),
        config.grpc,
        server.producer.clone(),
        token.clone(),
    ));

    tracing::info!("Ingest started successfully");

    while let Some(res) = supervisor.join_next().await {
        match res {
            Ok(Ok(())) => {
                info!("Server shutdown");
            }
            Ok(Err(err)) => {
                error!("Caught error: {err}");
            }
            Err(join_err) => {
                error!("Join error: {join_err}");
            }
        }
        server.token.cancel();
    }

    Ok(())
}
