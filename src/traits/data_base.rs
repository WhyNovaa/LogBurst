use crate::models::app::AuthCommandReceiver;
use crate::traits::start::Start;

pub trait DataBase: Start + Send + 'static {
    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self;
}