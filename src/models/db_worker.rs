use async_trait::async_trait;
use crate::models::app::AuthCommandReceiver;
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
        tokio::spawn(async move { self.auth.start().await; });
    }
}

#[async_trait]
impl<A: AuthRepository, L: LogsRepository> DataBase for DBWorker<A, L> {
    async fn new(auth_command_receiver: AuthCommandReceiver) -> Self {
        let auth = A::new(auth_command_receiver).await;

        let logs = L::new();

        Self {
            auth,
            logs,
        }
    }
}