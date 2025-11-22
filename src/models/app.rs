use std::marker::PhantomData;
use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::auth_command::AuthCommand;
use crate::models::log_command::LogCommand;
use crate::traits::client::Client;
use crate::traits::data_base::DataBase;
use crate::traits::logs_repository::LogsRepository;

pub type AuthCommandSender = mpsc::Sender<(AuthCommand, oneshot::Sender<Response>)>;
pub type AuthCommandReceiver = mpsc::Receiver<(AuthCommand, oneshot::Sender<Response>)>;

pub type LogCommandSender = mpsc::Sender<(LogCommand, oneshot::Sender<Response>)>;
pub type LogCommandReceiver = mpsc::Receiver<(LogCommand, oneshot::Sender<Response>)>;

const BUFFER_SIZE: usize = 1000;

pub struct App<C: Client<L>, D: DataBase, L: LogsRepository> {
    http_client: C,
    db: D,
    _marker: PhantomData<L>,
}

impl<C: Client<L>, D: DataBase, L: LogsRepository> App<C, D, L> {
    pub async fn new() -> Self {
        let (auth_command_sender, auth_command_receiver) = mpsc::channel::<(AuthCommand, oneshot::Sender<Response>)>(BUFFER_SIZE);

        let http_client = C::new(auth_command_sender);
        let db = D::new(auth_command_receiver).await;

        Self {
            http_client,
            db,
            _marker: PhantomData::default(),
        }
    }

    pub async fn start(self) {
        let (_db_res, _client_res) = tokio::join!(self.http_client.start(), self.db.start());
    }
}
