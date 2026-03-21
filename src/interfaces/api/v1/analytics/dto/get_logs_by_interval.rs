use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize)]
pub struct Filters {
    #[serde(flatten)]
    pub interval: Interval,
    pub service: Option<String>,
    pub level: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
pub struct Interval {
    #[serde(with = "time::serde::rfc3339")]
    pub from: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    pub to: OffsetDateTime,
}
