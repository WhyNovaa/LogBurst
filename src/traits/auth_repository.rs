use async_trait::async_trait;
use crate::models::app::AuthCommandReceiver;
use crate::models::user::User;
use crate::traits::start::Start;

#[async_trait]
pub trait AuthRepository: Start + Send + 'static {
    type Error: std::error::Error;

    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self;
    async fn create_user(&self, login: &str, password: &str, role_id: i32) -> Result<(), Self::Error>;
    async fn get_user_by_login(&self, login: &str) -> Result<Option<User>, Self::Error>;
}