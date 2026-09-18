use crate::config::Config;
use crate::writer::Writer;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod writer;

pub const LOG_TOPIC: &str = "logs";
pub const LOG_GROUP_ID: &str = "writer";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;

    let token = CancellationToken::new();

    let mut supervisor: JoinSet<anyhow::Result<()>> = JoinSet::new();

    for i in 0..cfg.kafka.partitions_number {
        let writer = Writer::new(cfg.kafka.clone(), cfg.clickhouse.clone(), token.clone()).await?;

        supervisor.spawn(writer.start_receiving());

        info!("Writer worker {i} started successfully");
    }

    info!("All writer workers started successfully");

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
        token.cancel();
    }

    Ok(())
}
