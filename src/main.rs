use crate::config::Config;
use crate::grpc::run_grpc_server;
use crate::rest::run_rest;
use crate::server::Server;
use std::sync::Arc;

mod db;
mod config;
mod rest;
mod server;
mod grpc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Arc::new(Config::from_env().unwrap());
    dbg!(&cfg);

    let server = Arc::new(Server::new(Arc::clone(&cfg)).await);

    let log_sender = server.logs_db.start_receiving(server.token.clone()).unwrap();

    let rest = tokio::spawn(run_rest(Arc::clone(&server), cfg.rest_cfg.clone(), log_sender.clone()));

    let grpc = tokio::spawn(run_grpc_server(Arc::clone(&server), cfg.grpc_config.clone(), log_sender));
    tokio::join!(rest, grpc);
}
