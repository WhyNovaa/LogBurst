use crate::models::http_client::api::handlers::auth::{AuthPayload, RegPayload};

#[derive(Debug)]
pub enum AuthCommand {
    CreateUser(RegPayload),
    Login(AuthPayload),
}
