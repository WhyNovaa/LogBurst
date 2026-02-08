use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Query {
    #[validate(range(min = 5, max = 15))]
    pub limit: u8,
}
