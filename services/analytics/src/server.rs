use crate::application::queries::clickhouse::AnalyticsQueries;
use crate::application::queries::postgres::UserQueries;
use storage::clickhouse::ClickhouseLog;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub struct Server {
    pub analytics_queries: AnalyticsQueries,
    pub user_queries: UserQueries,
    pub live_log_sender: broadcast::Sender<ClickhouseLog>,
    pub token: CancellationToken,
}
