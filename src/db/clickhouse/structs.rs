use crate::grpc::log_proto::LogEntry;
use chrono::Utc;
use clickhouse::Row;
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct Log {
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: OffsetDateTime,
    pub level: String,
    pub service: String,
    pub message: String,
    pub raw_data: String,
}

impl From<serde_json::Value> for Log {
    fn from(mut value: Value) -> Self {
        let mut take_field = |key: &str| -> Option<String> {
            value.as_object_mut().and_then(|map| map.remove(key)).and_then(|v| v.as_str().map(|s| s.to_string()))
        };

        Self {
            timestamp: time::OffsetDateTime::now_utc(),
            level: take_field("level").unwrap_or_else(|| "INFO".to_string()),
            service: take_field("service").unwrap_or_else(|| "unknown".to_string()),
            message: take_field("service").unwrap_or_default(),
            raw_data: value.to_string(),
        }
    }
}

impl From<LogEntry> for Log {
    fn from(value: LogEntry) -> Self {
        let proto_ts: Option<Timestamp> = value.timestamp;

        let timestamp = proto_ts
            .and_then(|ts| {
                OffsetDateTime::from_unix_timestamp(ts.seconds)
                    .ok()
                    .map(|odt| odt.replace_nanosecond(ts.nanos as u32).unwrap_or(odt))
            })
            .unwrap_or_else(OffsetDateTime::now_utc);

        Self {
            timestamp,
            level: value.level,
            service: value.service,
            message: value.message,
            raw_data: "".to_string(),
        }
    }
}