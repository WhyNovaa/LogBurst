use clickhouse::Row;
use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize, Row)]
pub struct LevelsCountIntervalBucket {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub time_bucket: OffsetDateTime,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}
