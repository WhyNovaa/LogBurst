use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuthBody {
    jwt: String,
}