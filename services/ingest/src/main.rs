use crate::interfaces::{run_grpc_server, run_rest};
use crate::security::keystore::KeyStore;
use crate::security::update_key_store;
use crate::server::Server;
use ::config::postgres::PostgresConfig;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_postgres::NoTls;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod interfaces;
mod security;
mod server;

pub const LOG_TOPIC: &str = "logs";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    service_utils::fmt::init_fmt();

    let config = config::Config::from_env()?;

    let token = CancellationToken::new();

    let server = Arc::new(Server::new(config.kafka, token.clone())?);

    let pool = connect_postgres(config.postgres).await?;

    let key_store = Arc::new(KeyStore::new(pool));

    tokio::spawn(service_utils::shutdown::wait_for_shutdown_signal(
        token.clone(),
    ));

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

async fn connect_postgres(pg_config: PostgresConfig) -> anyhow::Result<deadpool_postgres::Pool> {
    let mgr = Manager::from_config(
        pg_config.into(),
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    let pool = Pool::builder(mgr).max_size(2).build()?;

    Ok(pool)
}
