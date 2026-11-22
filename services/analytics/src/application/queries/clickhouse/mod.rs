use crate::infrastructure::clickhouse::repository::ClickhouseAnalyticsRepository;

mod get_levels_count;
mod get_levels_count_by_interval;
mod get_logs_by_interval;
mod get_services;

pub struct AnalyticsQueries {
    repo: ClickhouseAnalyticsRepository,
}

impl AnalyticsQueries {
    pub fn new(repo: ClickhouseAnalyticsRepository) -> Self {
        Self { repo }
    }
}
