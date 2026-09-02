use crate::config::clickhouse::ClickhouseConfig;
use crate::db::clickhouse::structs::{LevelsCountBucket, LevelsCountIntervalBucket, Log};
use clickhouse::Client;
use time::OffsetDateTime;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub mod structs;

const CHANNEL_SIZE: usize = 200_000;
const LIVE_CHANNEL_SIZE: usize = 15;
const INSERTER_MAX_ROWS: u64 = 50_000;
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
            .with_option("max_execution_time", "60");

        Self { client }
    }

    pub async fn get_logs_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        service_opt: Option<String>,
        level_opt: Option<String>,
        limit: u32,
        tx: async_channel::Sender<Log>,
    ) -> anyhow::Result<()> {
        let mut req = r#"
            SELECT
                ?fields
            FROM logs
            WHERE
                timestamp >= toDateTime64(?, 3, 'UTC') AND
                timestamp <= toDateTime64(?, 3, 'UTC')"#
            .to_string();

        if service_opt.is_some() {
            req.push_str(" AND service = ?");
        }
        if level_opt.is_some() {
            req.push_str(" AND level = ?");
        }

        req.push_str(" ORDER BY timestamp LIMIT ?");

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

        let limit = limit.min(5000);
        let query = query.bind(limit);

        let mut cursor = query.fetch::<Log>()?;
        while let Some(log) = cursor.next().await? {
            tx.send(log).await?;
        }

        Ok(())
    }

    pub async fn get_services(&self) -> anyhow::Result<Vec<String>> {
        const REQ: &str = r#"
            SELECT DISTINCT service
            FROM logs
        "#;

        Ok(self.client.query(REQ).fetch_all().await?)
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
                toDateTime64(?, 3, 'UTC') AS end_time,
                toDateTime64(toStartOfHour(start_time), 3, 'UTC') AS start_bucket,
                toDateTime64(toStartOfHour(end_time), 3, 'UTC') AS end_bucket
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
                FROM  start_bucket
                TO    end_bucket
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
    ) -> anyhow::Result<(
        async_channel::Sender<Log>,
        tokio::sync::broadcast::Sender<Log>,
    )> {
        let mut inserter = self.client.inserter::<Log>("logs")?;

        let (tx, rx) = async_channel::bounded::<Log>(CHANNEL_SIZE);
        let (live_tx, _) = tokio::sync::broadcast::channel::<Log>(LIVE_CHANNEL_SIZE);
        let mut tick_interval = interval(FLUSH_INTERVAL);
        tokio::spawn({
            let live_tx = live_tx.clone();
            async move {
                loop {
                    tokio::select! {
                        msg = rx.recv() => {
                            match msg {
                                Ok(log) => {
                                    if let Err(e) = inserter.write(&log) {
                                        error!("Error serializing log: {}", e);
                                        continue;
                                    }

                                    if rand::random_range(1..=100) == 100 {
                                        if let Err(e) = live_tx.send(log) {
                                            error!("Error sending log into live channel: {}", e);
                                        }
                                    }

                                    let pending_amount = inserter.pending().rows;

                                    if pending_amount >= INSERTER_MAX_ROWS {
                                        info!("Batch limit reached, committing {} logs...", pending_amount);
                                        match inserter.force_commit().await {
                                            Ok(res) => {
                                                info!("{} rows were written", res.rows);
                                                tick_interval.reset();
                                            }
                                            Err(e) => error!("Failed to commit batch: {}", e),
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to receive log: {}", e);
                                    break;
                                }
                            }
                        }

                        _ = tick_interval.tick() => {
                            let pending_amount = inserter.pending().rows;

                            if pending_amount > 0 {
                                info!("Time interval, committing {} logs...", pending_amount);
                                match inserter.force_commit().await {
                                    Ok(res) => {
                                        info!("{} rows were written", res.rows);
                                        tick_interval.reset();
                                    }
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
                            if let Err(e) = inserter.force_commit().await {
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
