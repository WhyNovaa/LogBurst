use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
pub struct IntervalQuery {
    #[serde(with = "time::serde::rfc3339")]
    pub from: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    pub to: OffsetDateTime,
}

#[derive(Serialize)]
pub struct ErrorBucket {
    pub time_bucket: OffsetDateTime,
    pub error_count: u64,
    pub warning_count: u64,
    pub info_count: u64,
}
impl ErrorBucket {
    pub fn from_ref(err_bucket: &crate::db::clickhouse::structs::IntervalInfo) -> Self {
        Self {
            time_bucket: err_bucket.time_bucket,
            error_count: err_bucket.error_count,
            warning_count: err_bucket.warning_count,
            info_count: err_bucket.info_count,
        }
    }
}
