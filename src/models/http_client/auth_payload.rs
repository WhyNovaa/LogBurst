use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub login: String,
    pub password: String,
}