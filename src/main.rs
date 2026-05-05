use crate::config::Config;
use crate::interfaces::api::run_rest;
use crate::interfaces::grpc::run_grpc_server;
use crate::security::keystore::KeyStore;
use crate::security::update_key_store;
use crate::server::Server;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod db;
mod interfaces;
mod security;
mod server;

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

    let cfg = Arc::new(Config::from_env()?);
    dbg!(&cfg);

    let server = Arc::new(Server::new(Arc::clone(&cfg)).await);

    let (log_sender, live_log_receiver) = server.logs_db.start_receiving(server.token.clone())?;

    let key_store = Arc::new(KeyStore::new(KeyStore::load()?));

    let mut supervisor: JoinSet<anyhow::Result<()>> = JoinSet::new();

    supervisor.spawn(update_key_store(server.token.clone(), key_store.clone()));

    // Rest
    supervisor.spawn(run_rest(
        Arc::clone(&server),
        cfg.rest_cfg.clone(),
        log_sender.clone(),
        live_log_receiver,
        key_store,
    ));

    // gRPC
    supervisor.spawn(run_grpc_server(
        Arc::clone(&server),
        cfg.grpc_config.clone(),
        log_sender,
        server.token.clone(),
    ));

    while let Some(res) = supervisor.join_next().await {
        match res {
            Ok(Ok(())) => {
                info!("Server shutdown");
            }
            Ok(Err(err)) => {
                error!("Catched error: {err}");
            }
            Err(join_err) => {
                error!("Join error: {join_err}");
            }
        }
        server.token.cancel();
    }

    Ok(())
}
