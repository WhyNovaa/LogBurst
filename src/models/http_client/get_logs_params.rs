use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetLogsParams {
    pub service: Option<String>,
    pub level: Option<String>,
}