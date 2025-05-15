use sqlx::FromRow;
use crate::models::http_client::role::Role;

#[derive(Debug)]
pub struct User {
    pub login: String,
    pub hashed_password: String,
    pub role: Role,
}

#[derive(Debug, FromRow)]
pub struct DbUser {
    pub login: String,
    pub hashed_password: String,
    pub role_name: String,
}