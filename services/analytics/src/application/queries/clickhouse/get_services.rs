use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::infrastructure::error::DbError;

impl AnalyticsQueries {
    pub async fn get_services(&self) -> Result<Vec<String>, DbError> {
        self.repo.get_services().await
    }
}
