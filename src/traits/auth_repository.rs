use crate::db::pg::AuthRepositoryError;
use crate::models::app::AuthCommandReceiver;
use crate::models::http_client::api::handlers::auth::RegPayload;
use crate::traits::start::Start;

pub trait AuthRepository: Start + Send + 'static {
    type Error: std::error::Error;

    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self;
    async fn create_user(&self, payload: RegPayload) -> Result<(), Self::Error>;
}