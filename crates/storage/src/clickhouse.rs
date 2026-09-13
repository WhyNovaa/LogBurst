use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct ClickhouseLog {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
    pub level: String,
    pub service: String,
    pub message: String,
    pub raw_data: String,
}

impl From<domain::Log> for ClickhouseLog {
    fn from(value: domain::Log) -> Self {
        Self {
            timestamp: value.timestamp,
            level: value.level,
            service: value.service,
            message: value.message,
            raw_data: value.raw_data,
        }
    }
}
