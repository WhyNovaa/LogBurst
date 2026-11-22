use clickhouse::Row;
use serde::Deserialize;

#[derive(Deserialize, Row)]
pub struct LevelsCountBucket {
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}
