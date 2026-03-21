use serde::Serialize;

#[derive(Serialize)]
pub struct LevelsCountBucket {
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
}

impl From<crate::db::clickhouse::structs::LevelsCountBucket> for LevelsCountBucket {
    fn from(value: crate::db::clickhouse::structs::LevelsCountBucket) -> Self {
        Self {
            error_count: value.error_count,
            warn_count: value.warn_count,
            info_count: value.info_count,
        }
    }
}
