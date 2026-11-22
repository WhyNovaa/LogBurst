use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::infrastructure::clickhouse::rows::levels_count_interval::LevelsCountIntervalBucket;
use crate::infrastructure::error::DbError;
use time::OffsetDateTime;

impl AnalyticsQueries {
    pub async fn get_levels_count_by_interval(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<LevelsCountIntervalBucket>, DbError> {
        self.repo.get_levels_count_by_interval(from, to).await
    }
}
