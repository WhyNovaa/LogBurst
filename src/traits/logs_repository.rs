use clickhouse::Client;
use crate::models::app::LogCommandReceiver;
use crate::models::log::Log;
use crate::traits::start::Start;

pub trait LogsRepository: Start + Send + 'static {
    type Error: std::error::Error;
    fn new(log_receiver: LogCommandReceiver) -> Self;
    async fn save_logs(client: &Client, log: &[Log]) -> Result<(), Self::Error>;
    async fn get_logs(&self, service: Option<String>, level: Option<String>) -> Result<Vec<Log>, Self::Error>;
}