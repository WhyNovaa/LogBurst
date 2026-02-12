use crate::config::clickhouse::ClickhouseConfig;
use crate::db::clickhouse::structs::{LevelsCountBucket, LevelsCountIntervalBucket, Log};
use clickhouse::Client;
use kanal::AsyncSender;
use time::OffsetDateTime;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub mod structs;

const CHANNEL_SIZE: usize = 200_000;
const LIVE_CHANNEL_SIZE: usize = 15;
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

    pub async fn get_logs_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        service_opt: Option<String>,
        level_opt: Option<String>,
        tx: kanal::AsyncSender<Log>,
    ) -> anyhow::Result<()> {
        let mut req= r#"
            SELECT
                ?fields
            FROM logs
            WHERE
                timestamp >= toDateTime64(?, 3, 'UTC') AND
                timestamp <= toDateTime64(?, 3, 'UTC')"#.to_string();

        if service_opt.is_some() {
            req.push_str(" AND service = ?");
        }
        if level_opt.is_some() {
            req.push_str(" AND level = ?");
        }

        let from = from.to_utc().unix_timestamp();
        let to = to.to_utc().unix_timestamp();
        
        info!("Querying logs from {} to {}", from, to);

        let mut query = self.client.query(req.as_str()).bind(from).bind(to);

        if let Some(service) = service_opt {
            query = query.bind(service);
        }
        if let Some(level) = level_opt {
            query = query.bind(level);
        }

        let mut cursor = query.fetch::<Log>()?;
        while let Some(log) = cursor.next().await? {
            tx.send(log).await?;
        }

        Ok(())
    }

    pub async fn get_levels_count(&self) -> anyhow::Result<LevelsCountBucket> {
        const REQ: &str = r#"
            SELECT 
                countIf(level = 'error') as error_count,
                countIf(level = 'warn') as warn_count,
                countIf(level = 'info') as info_count
            FROM logs"#;

        Ok(self
            .client
            .query(REQ)
            .fetch_one::<LevelsCountBucket>()
            .await?)
    }

    pub async fn get_levels_count_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> anyhow::Result<Vec<LevelsCountIntervalBucket>> {
        const REQ: &str = r#"
            WITH
                toDateTime64(?, 3, 'UTC') AS start_time,
                toDateTime64(?, 3, 'UTC') AS end_time
            SELECT
                toDateTime64(toStartOfInterval(timestamp, INTERVAL 1 HOUR), 3, 'UTC') AS time_bucket,
                countIf(level = 'error') AS error_count,
                countIf(level = 'warn') AS warn_count,
                countIf(level = 'info') AS info_count
            FROM logs
            WHERE
                timestamp >= start_time
                AND timestamp <=  end_time
            GROUP BY time_bucket
            ORDER BY time_bucket
            WITH FILL
                FROM  start_time
                TO    end_time
            STEP INTERVAL 1 HOUR
        "#;

        let from = from.to_utc().unix_timestamp();
        let to = to.to_utc().unix_timestamp();

        Ok(self
            .client
            .query(&REQ)
            .bind(&from)
            .bind(&to)
            .fetch_all::<LevelsCountIntervalBucket>()
            .await?)
    }

    pub fn start_receiving(
        &self,
        token: CancellationToken,
    ) -> anyhow::Result<(AsyncSender<Log>, tokio::sync::broadcast::Sender<Log>)> {
        let mut inserter = self
            .client
            .inserter::<Log>("logs")?
            .with_max_rows(INSERTER_MAX_ROWS)
            .with_max_bytes(INSERTER_MAX_BYTES);

        let (tx, rx) = kanal::bounded_async::<Log>(CHANNEL_SIZE);
        let (live_tx, _) = tokio::sync::broadcast::channel::<Log>(LIVE_CHANNEL_SIZE);
        let mut tick_interval = interval(FLUSH_INTERVAL);

        tokio::spawn({
            let live_tx = live_tx.clone();
            async move {
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

                                    if rand::random_range(1..=100) == 100 {
                                        if let Err(e) = live_tx.send(log) {
                                            error!("Error sending log into live channel: {}", e);
                                        }
                                    }

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
                                Err(_) => {}
                            }
                        }

                        _ = tick_interval.tick() => {
                            if pending_count > 0 {
                                info!("Time interval, committing {} logs...", pending_count);
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
            }
        });

        Ok((tx, live_tx))
    }
}
