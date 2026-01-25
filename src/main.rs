use crate::config::Config;
use crate::grpc::run_grpc_server;
use crate::rest::run_rest;
use crate::server::Server;
use std::sync::Arc;

mod config;
mod db;
mod grpc;
mod rest;
mod server;

use tracing_subscriber::{EnvFilter, fmt};

pub fn init_fmt() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info").add_directive("clickhouse=debug".parse().unwrap())
    });

    fmt()
        .with_env_filter(env_filter)
        .with_target(true) // показывает crate/module
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

#[tokio::main]
async fn main() {
    init_fmt();

    let cfg = Arc::new(Config::from_env().unwrap());
    dbg!(&cfg);

    let server = Arc::new(Server::new(Arc::clone(&cfg)).await);

    let log_sender = server
        .logs_db
        .start_receiving(server.token.clone())
        .unwrap();

    let rest = tokio::spawn(run_rest(
        Arc::clone(&server),
        cfg.rest_cfg.clone(),
        log_sender.clone(),
    ));

    let grpc = tokio::spawn(run_grpc_server(
        Arc::clone(&server),
        cfg.grpc_config.clone(),
        log_sender,
    ));

    tokio::join!(rest, grpc);
}
