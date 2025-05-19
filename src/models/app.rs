use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::auth_command::AuthCommand;
use crate::models::log_command::LogCommand;
use crate::traits::client::Client;
use crate::traits::data_base::DataBase;

pub type AuthCommandSender = mpsc::Sender<(AuthCommand, oneshot::Sender<Response>)>;
pub type AuthCommandReceiver = mpsc::Receiver<(AuthCommand, oneshot::Sender<Response>)>;

pub type LogCommandSender = mpsc::Sender<(LogCommand, oneshot::Sender<Response>)>;
pub type LogCommandReceiver = mpsc::Receiver<(LogCommand, oneshot::Sender<Response>)>;

const BUFFER_SIZE: usize = 100;

pub struct App<C: Client, D: DataBase> {
    http_client: C,
    db: D,
}

impl<C: Client, D: DataBase> App<C, D> {
    pub async fn new() -> Self {
        let (auth_command_sender, auth_command_receiver) = mpsc::channel::<(AuthCommand, oneshot::Sender<Response>)>(BUFFER_SIZE);
        let (log_command_sender, log_command_receiver) = mpsc::channel::<(LogCommand, oneshot::Sender<Response>)>(BUFFER_SIZE);

        let http_client = C::new(auth_command_sender, log_command_sender);
        let db = D::new(auth_command_receiver, log_command_receiver).await;

        Self {
            http_client,
            db,
        }
    }

    pub async fn start(self) {
        let client_task = tokio::spawn(async move {
            self.http_client.start().await;
        });
        let db_task = tokio::spawn(async move {
            self.db.start().await;
        });

        let (_db_res, _client_res) = tokio::join!(db_task, client_task);
    }
}
