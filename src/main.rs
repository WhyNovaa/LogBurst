use crate::config::Config;
use crate::rest::run_rest;
use crate::server::Server;
use std::sync::Arc;

mod db;
mod config;
mod rest;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Arc::new(Config::from_env().unwrap());
    dbg!(&cfg);

    let server = Arc::new(Server::new(Arc::clone(&cfg)).await);

    let log_sender = server.logs_db.start_receiving(server.token.clone()).unwrap();

    let rest = tokio::spawn(run_rest(server, cfg, log_sender));

    tokio::join!(rest);
}
