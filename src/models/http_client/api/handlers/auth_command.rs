use crate::models::http_client::api::handlers::auth::{AuthPayload, RegPayload, Role};

#[derive(Debug)]
pub enum AuthCommand {
    CreateUser {
        login: String,
        password: String,
        role: Role,
    },
    Login {
        login: String,
        password: String,
    },
}
