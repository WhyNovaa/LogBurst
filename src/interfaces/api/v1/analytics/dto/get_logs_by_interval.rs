use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
pub struct IntervalQuery {
    #[serde(with = "time::serde::rfc3339")]
    pub from: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    pub to: OffsetDateTime,
}
