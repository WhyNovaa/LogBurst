use crate::config::Config;
use crate::writer::Writer;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

mod config;
mod writer;

pub const LOG_TOPIC: &str = "logs";
pub const LOG_GROUP_ID: &str = "writer";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    service_utils::fmt::init_fmt();

    let cfg = Config::from_env()?;

    let token = CancellationToken::new();

    tokio::spawn(service_utils::shutdown::wait_for_shutdown_signal(
        token.clone(),
    ));

    let res = tokio::spawn({
        let writer = Writer::new(cfg.kafka.clone(), cfg.clickhouse.clone(), token.clone()).await?;
        writer.start_writing()
    });

    info!("Writer started successfully");

    match res.await {
        Ok(Ok(())) => {
            info!("Worker shutdown");
        }
        Ok(Err(err)) => {
            error!("Caught error: {err}");
        }
        Err(join_err) => {
            error!("Join error: {join_err}");
        }
    }

    Ok(())
}
