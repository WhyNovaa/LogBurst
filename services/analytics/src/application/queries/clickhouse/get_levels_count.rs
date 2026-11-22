use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::infrastructure::clickhouse::rows::levels_count::LevelsCountBucket;
use crate::infrastructure::error::DbError;

impl AnalyticsQueries {
    pub async fn get_levels_count(&self) -> Result<LevelsCountBucket, DbError> {
        self.repo.get_levels_count().await
    }
}
