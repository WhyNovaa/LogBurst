use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Log {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
}