use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::infrastructure::error::DbError;
use clickhouse::query::RowCursor;
use storage::clickhouse::ClickhouseLog;
use time::OffsetDateTime;

impl AnalyticsQueries {
    pub fn get_logs_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        service: Option<String>,
        level: Option<String>,
        limit: u32,
    ) -> Result<RowCursor<ClickhouseLog>, DbError> {
        self.repo
            .get_logs_by_interval(from, to, service, level, limit)
    }
}
