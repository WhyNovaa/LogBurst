use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegPayload {
    pub login: String,
    pub password: String,
}