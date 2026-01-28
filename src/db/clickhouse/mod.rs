pub mod structs;

use crate::config::clickhouse::ClickhouseConfig;
use crate::db::clickhouse::structs::Log;
use clickhouse::Client;
use kanal::AsyncSender;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

const CHANNEL_SIZE: usize = 200_000;
const INSERTER_MAX_ROWS: u64 = 50_000;
const INSERTER_MAX_BYTES: u64 = 10 * 1024 * 1024;
const FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

pub struct ClickHouse {
    client: Client,
}

impl ClickHouse {
    pub async fn connect(cfg: ClickhouseConfig) -> Self {
        let client = Client::default()
            .with_url(cfg.url())
            .with_user(cfg.user)
            .with_password(cfg.password)
            .with_option("max_execution_time", "60")
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0");

        Self { client }
    }

    pub fn start_receiving(&self, token: CancellationToken) -> anyhow::Result<AsyncSender<Log>> {
        let mut inserter = self
            .client
            .inserter::<Log>("logs")?
            .with_max_rows(INSERTER_MAX_ROWS)
            .with_max_bytes(INSERTER_MAX_BYTES);

        let (tx, rx) = kanal::bounded_async(CHANNEL_SIZE);

        let mut tick_interval = interval(FLUSH_INTERVAL);

        tokio::spawn(async move {
            let mut pending_count = 0;

            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Ok(log) => {
                                if let Err(e) = inserter.write(&log) {
                                    error!("Error serializing log: {}", e);
                                    continue;
                                }
                                pending_count += 1;

                                if pending_count >= INSERTER_MAX_ROWS {
                                    info!("Batch limit reached, committing {} logs...", pending_count);
                                    match inserter.commit().await {
                                        Ok(_) => {
                                            pending_count = 0;
                                            tick_interval.reset();
                                        }
                                        Err(e) => error!("Failed to commit batch: {}", e),
                                    }
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }

                    _ = tick_interval.tick() => {
                        if pending_count > 0 {
                            debug!("Time interval, committing {} logs...", pending_count);
                            match inserter.commit().await {
                                Ok(_) => pending_count = 0,
                                Err(e) => error!("Failed to commit by timer: {}", e),
                            }
                        }
                    }

                    _ = token.cancelled() => {
                        warn!("Shutdown signal received. Draining logs...");

                        let _ = rx.close();

                        while let Ok(log) = rx.recv().await {
                            if let Err(e) = inserter.write(&log) {
                                info!("Error writing remaining log: {}", e);
                            }
                        }
                        if let Err(e) = inserter.commit().await {
                            error!("Failed to commit logs on shutdown: {}", e)
                        }

                        info!("Draining complete.");
                        break;
                    }
                }
            }
        });

        Ok(tx)
    }
}
