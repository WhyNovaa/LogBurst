use crate::db::pg::AuthRepositoryError;
use crate::models::http_client::api::handlers::auth::RegPayload;
use crate::traits::new::AsyncNew;

pub trait AuthRepository: AsyncNew {
    type Error: std::error::Error;
    async fn create_user(&self, payload: RegPayload) -> Result<(), Self::Error>;
}