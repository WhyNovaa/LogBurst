use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Deserialize)]
pub struct IntervalQuery {
    pub from: OffsetDateTime,
    pub to: OffsetDateTime,
}
