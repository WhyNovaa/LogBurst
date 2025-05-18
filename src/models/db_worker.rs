use async_trait::async_trait;
use crate::models::app::{AuthCommandReceiver, LogCommandReceiver};
use crate::traits::auth_repository::AuthRepository;
use crate::traits::logs_repository::LogsRepository;
use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

pub struct DBWorker<A: AuthRepository, L: LogsRepository> {
    auth: A,
    logs: L
}

#[async_trait]
impl<A: AuthRepository, L: LogsRepository> Start for DBWorker<A, L> {
    async fn start(self) {

        tokio::spawn(async move {
            self.auth.start().await;
        });

        tokio::spawn(async move {
            self.logs.start().await;
        });
    }
}

#[async_trait]
impl<A: AuthRepository, L: LogsRepository> DataBase for DBWorker<A, L> {
    async fn new(auth_command_receiver: AuthCommandReceiver, log_command_receiver: LogCommandReceiver) -> Self {
        let auth = A::new(auth_command_receiver).await;

        let logs = L::new(log_command_receiver);

        Self {
            auth,
            logs,
        }
    }
}