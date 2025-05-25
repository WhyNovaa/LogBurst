use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Row, Serialize, Deserialize, Debug)]
pub struct Log {
    pub timestamp: u32,
    pub level: String,
    pub service: String,
    pub message: String,
}