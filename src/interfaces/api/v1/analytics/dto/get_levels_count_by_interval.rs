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
pub struct LevelsCountIntervalBucket {
    pub time_bucket: OffsetDateTime,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}
impl LevelsCountIntervalBucket {
    pub fn from_ref(
        err_bucket: &crate::db::clickhouse::structs::LevelsCountIntervalBucket,
    ) -> Self {
        Self {
            time_bucket: err_bucket.time_bucket,
            error_count: err_bucket.error_count,
            warn_count: err_bucket.warn_count,
            info_count: err_bucket.info_count,
        }
    }
}
