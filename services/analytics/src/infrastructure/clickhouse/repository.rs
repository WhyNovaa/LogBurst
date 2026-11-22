pub(crate) use crate::infrastructure::clickhouse::rows::levels_count::LevelsCountBucket;
use crate::infrastructure::clickhouse::rows::levels_count_interval::LevelsCountIntervalBucket;
use crate::infrastructure::error::DbError;
use clickhouse::query::RowCursor;
use storage::clickhouse::ClickhouseLog;
use time::OffsetDateTime;

type Result<T> = std::result::Result<T, DbError>;

pub struct ClickhouseAnalyticsRepository {
    client: clickhouse::Client,
}

impl ClickhouseAnalyticsRepository {
    pub fn new(client: clickhouse::Client) -> Self {
        Self { client }
    }

    pub fn get_logs_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        service: Option<String>,
        level: Option<String>,
        limit: u32,
    ) -> Result<RowCursor<ClickhouseLog>> {
        let mut sql = r#"
            SELECT ?fields
            FROM logs
            WHERE
                timestamp >= toDateTime64(?, 3, 'UTC') AND
                timestamp <= toDateTime64(?, 3, 'UTC')"#
            .to_string();

        if service.is_some() {
            sql.push_str(" AND service = ?");
        }
        if level.is_some() {
            sql.push_str(" AND level = ?");
        }
        sql.push_str(" ORDER BY timestamp LIMIT ?");

        let mut query = self
            .client
            .query(&sql)
            .bind(from.to_utc().unix_timestamp())
            .bind(to.to_utc().unix_timestamp());

        if let Some(service) = service {
            query = query.bind(service);
        }
        if let Some(level) = level {
            query = query.bind(level);
        }

        Ok(query.bind(limit.min(5000)).fetch::<ClickhouseLog>()?)
    }

    pub async fn get_levels_count(&self) -> Result<LevelsCountBucket> {
        const QUERY: &str = r#"
            SELECT
                countIf(level = 'error') as error_count,
                countIf(level = 'warn') as warn_count,
                countIf(level = 'info') as info_count
            FROM logs"#;

        Ok(self
            .client
            .query(QUERY)
            .fetch_one::<LevelsCountBucket>()
            .await?)
    }

    pub async fn get_levels_count_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<LevelsCountIntervalBucket>> {
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
            .query(REQ)
            .bind(from)
            .bind(to)
            .fetch_all::<LevelsCountIntervalBucket>()
            .await?)
    }

    pub async fn get_services(&self) -> Result<Vec<String>> {
        const REQ: &str = r#"
            SELECT DISTINCT service
            FROM logs
        "#;

        Ok(self.client.query(REQ).fetch_all().await?)
    }
}
