use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Row)]
pub struct Log {
    pub timestamp: u32,
    pub level: String,
    pub service: String,
    pub message: String,
}