use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreationPayload {
    pub login: String,
    pub password: String,
    pub role_name: String,
}
