use crate::interfaces::grpc::log_proto::{LogEntry, LogLevel};
use anyhow::anyhow;
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use tonic::Status;
use validator::{Validate, ValidationError};

#[derive(Validate, Debug, Clone, Serialize, Deserialize, Row)]
pub struct Log {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: OffsetDateTime,
    #[validate(custom(function = "validate_level"))]
    pub level: String,
    pub service: String,
    pub message: String,
    pub raw_data: String,
}

fn validate_level(level: &str) -> Result<(), ValidationError> {
    match level {
        "info" | "warn" | "error" => Ok(()),
        _ => Err(ValidationError::new("Invalid level")),
    }
}
impl From<LogLevel> for String {
    fn from(value: LogLevel) -> Self {
        let res = match value {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };

        res.to_string()
    }
}

impl TryFrom<serde_json::Value> for Log {
    type Error = anyhow::Error;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        let map = value.as_object_mut().ok_or(anyhow!("Value must be JSON"))?;

        let mut take_field = |key: &str| -> Option<String> {
            map.remove(key)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        };

        Ok(Self {
            timestamp: time::OffsetDateTime::now_utc(),
            level: take_field("level").ok_or(anyhow::Error::msg("field level not found in log"))?,
            service: take_field("service")
                .ok_or(anyhow::Error::msg("field service not found in log"))?,
            message: take_field("message")
                .ok_or(anyhow::Error::msg("field message not found in log"))?,
            raw_data: value.to_string(),
        })
    }
}

impl TryFrom<LogEntry> for Log {
    type Error = tonic::Status;

    fn try_from(value: LogEntry) -> Result<Self, Self::Error> {
        let proto_ts = value
            .timestamp
            .ok_or_else(|| tonic::Status::invalid_argument("missing timestamp in log"))?;
        let nanos = u32::try_from(proto_ts.nanos)
            .map_err(|_| Status::invalid_argument("wrong timestamp"))?;
        let timestamp = OffsetDateTime::from_unix_timestamp(proto_ts.seconds)
            .and_then(|odt| odt.replace_nanosecond(nanos))
            .map_err(|_| Status::invalid_argument("wrong timestamp"))?;

        Ok(Self {
            timestamp,
            level: value.level().into(),
            service: value.service,
            message: value.message,
            raw_data: "".to_string(),
        })
    }
}

#[derive(Deserialize, Row)]
pub struct LevelsCountIntervalBucket {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub time_bucket: OffsetDateTime,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}

#[derive(Deserialize, Row)]
pub struct LevelsCountBucket {
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}
