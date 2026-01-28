use crate::config::Config;
use crate::interfaces::api::run_rest;
use crate::interfaces::grpc::run_grpc_server;
use crate::security::keystore::KeyStore;
use crate::security::update_key_store;
use crate::server::Server;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt};

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

    let log_sender = server.logs_db.start_receiving(server.token.clone())?;

    let key_store = Arc::new(KeyStore::new(KeyStore::load()?));

    let update_hashers_task = tokio::spawn(update_key_store(key_store.clone()));

    let rest_task = tokio::spawn(run_rest(
        Arc::clone(&server),
        cfg.rest_cfg.clone(),
        log_sender.clone(),
        key_store,
    ));

    let grpc_task = tokio::spawn(run_grpc_server(
        Arc::clone(&server),
        cfg.grpc_config.clone(),
        log_sender,
    ));

    tokio::join!(update_hashers_task, rest_task, grpc_task);

    Ok(())
}
