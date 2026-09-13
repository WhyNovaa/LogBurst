use crate::config::Config;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod writer;

pub const LOG_TOPIC: &str = "logs";
pub const LOG_GROUP_ID: &str = "writer";

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
    let cfg = Config::from_env()?;

    let token = CancellationToken::new();

    let mut supervisor: JoinSet<anyhow::Result<()>> = JoinSet::new();

    for _ in 0..cfg.kafka.partitions_number {
        let consumer = cfg.kafka.create_consumer(LOG_GROUP_ID)?;
    }

    Ok(())
}
