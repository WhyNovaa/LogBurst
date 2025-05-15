use serde::{Deserialize, Serialize};
use crate::models::http_client::role::Role;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub login: String,
    pub role: Role,
    pub exp: usize,
}