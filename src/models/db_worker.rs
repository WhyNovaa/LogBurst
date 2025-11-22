use async_trait::async_trait;
use crate::models::app::{AuthCommandReceiver, LogCommandReceiver};
use crate::traits::auth_repository::AuthRepository;
use crate::traits::logs_repository::LogsRepository;
use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

pub struct DBWorker<A: AuthRepository> {
    auth: A,
}

#[async_trait]
impl<A: AuthRepository> Start for DBWorker<A> {
    async fn start(self) {

        tokio::spawn(async move {
            self.auth.start().await;
        });
    }
}

#[async_trait]
impl<A: AuthRepository> DataBase for DBWorker<A> {
    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self {
        let auth = A::new(auth_command_receiver).await;

        Self {
            auth,
        }
    }
}