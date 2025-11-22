use async_trait::async_trait;
use clickhouse::Client;
use tokio::sync::mpsc::UnboundedReceiver;
use crate::models::app::LogCommandReceiver;
use crate::models::log::Log;
use crate::traits::start::Start;

#[async_trait]
pub trait LogsRepository: Send + Sync + 'static {
    type Error: std::error::Error;
    fn new() -> Self;
    async fn save_logs(&self, logs: &Vec<Log>) -> Result<(), Self::Error>;

    //todo remove
    async fn save_log(&self, logs: &Log) -> Result<(), Self::Error>;
    async fn get_logs(&self, service: Option<String>, level: Option<String>) -> Result<Vec<Log>, Self::Error>;
}