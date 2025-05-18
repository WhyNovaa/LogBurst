use clickhouse::Row;
use serde::Deserialize;

#[derive(Deserialize, Debug, Row)]
pub struct Log {
    pub timestamp: i64,
    pub level: String,
    pub service: String,
    pub message: String,
}