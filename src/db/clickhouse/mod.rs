pub mod structs;

use crate::config::clickhouse::ClickhouseConfig;
use crate::db::clickhouse::structs::Log;
use clickhouse::Client;
use kanal::AsyncSender;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct ClickHouse {
    client: Client,
}

impl ClickHouse {
    pub async fn connect(cfg: ClickhouseConfig) -> Self {
        let client = Client::default()
            .with_url("http://clickhouse-server:8123")
            .with_user(cfg.user)
            .with_password(cfg.password)
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0")
            .with_option("max_execution_time", "60");

        Self {
            client
        }
    }

    pub fn start_receiving(&self, token: CancellationToken) -> anyhow::Result<AsyncSender<Log>> {
        let mut inserter = self.client.inserter::<Log>("logs")?.with_max_rows(50000).with_max_bytes(10 * 1024 * 1024).with_period(Some(std::time::Duration::from_secs(300)));
        let (tx, rx) = kanal::bounded_async(200_000);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Ok(log) => {
                                if let Err(e) = inserter.write(&log) {
                                    error!("Error writing log to buffer: {}", e);
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }

                    _ = token.cancelled() => {
                        warn!("Shutdown signal received. Draining logs...");

                        rx.close();

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