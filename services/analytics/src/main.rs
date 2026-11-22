use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::application::queries::postgres::UserQueries;
use crate::config::Config;
use crate::infrastructure::clickhouse::repository::ClickhouseAnalyticsRepository;
use crate::infrastructure::kafka::LiveLogs;
use crate::infrastructure::postgres::repository::PostgresUserRepository;
use crate::server::Server;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

pub mod application;
mod config;
pub mod infrastructure;
mod interfaces;
pub mod security;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    service_utils::fmt::init_fmt();

    let config = Config::from_env()?;

    let token = CancellationToken::new();

    let (live_log_sender, _) = broadcast::channel(15);

    let live_logs = LiveLogs::new(config.kafka, live_log_sender.clone(), token.clone())?;

    let server = Arc::new(Server {
        analytics_queries: AnalyticsQueries::new(ClickhouseAnalyticsRepository::new(
            config.clickhouse.client(),
        )),
        user_queries: UserQueries::new(PostgresUserRepository::new(config.postgres.create_pool()?)),
        live_log_sender,
        token: token.clone(),
    });

    let shutdown = tokio::spawn(service_utils::shutdown::wait_for_shutdown_signal(
        token.clone(),
    ));

    let mut supervisor = JoinSet::new();

    supervisor.spawn(interfaces::http::run(server, config.http, config.jwt));

    supervisor.spawn(live_logs.run());

    tracing::info!("Analytics started successfully");

    let mut failure = None;
    while let Some(result) = supervisor.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                failure.get_or_insert(error);
            }
            Err(error) => {
                failure.get_or_insert(error.into());
            }
        }
        token.cancel();
    }

    shutdown.abort();

    if let Some(error) = failure {
        return Err(error);
    }

    Ok(())
}
