use async_trait::async_trait;
use crate::models::app::{AuthCommandReceiver, LogCommandReceiver};
use crate::traits::start::Start;

#[async_trait]
pub trait DataBase: Start + Send + 'static {
    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self;
}