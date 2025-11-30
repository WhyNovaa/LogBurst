use crate::config::Config;
use crate::server::Server;

mod db;
mod config;
mod rest;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env().unwrap();
    dbg!(&cfg);

    let server = Server::new(cfg).await;
}
