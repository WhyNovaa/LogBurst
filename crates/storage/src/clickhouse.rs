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
    pub kafka_partition: i32,
    pub kafka_offset: i64,
}

impl ClickhouseLog {
    pub fn from_kafka(value: domain::Log, partition: i32, offset: i64) -> Self {
        Self {
            timestamp: value.timestamp,
            level: value.level,
            service: value.service,
            message: value.message,
            raw_data: value.raw_data,
            kafka_partition: partition,
            kafka_offset: offset,
        }
    }
}
