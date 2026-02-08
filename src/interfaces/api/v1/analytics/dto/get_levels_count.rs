use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    pub count: u64,
}
