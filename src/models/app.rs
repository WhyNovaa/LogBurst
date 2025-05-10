use axum::response::Response;
use sqlx::Database;
use tokio::sync::{mpsc, oneshot};
use crate::models::http_client::api::handlers::auth_command::AuthCommand;
use crate::models::log::Log;
use crate::traits::client::Client;
use crate::traits::data_base::DataBase;
use crate::traits::start::Start;

pub type AuthCommandSender = mpsc::Sender<(AuthCommand, oneshot::Sender<Response>)>;
pub type AuthCommandReceiver = mpsc::Receiver<(AuthCommand, oneshot::Sender<Response>)>;

pub type LogSender = mpsc::Sender<(Log, oneshot::Sender<Response>)>;
pub type LogReceiver = mpsc::Receiver<(Log, oneshot::Sender<Response>)>;


const BUFFER_SIZE: usize = 100;

pub struct App<C: Client, D: DataBase> {
    http_client: C,
    db: D,
}

impl<C: Client, D: DataBase> App<C, D> {
    pub async fn new() -> Self {
        let (c_s, c_r) = mpsc::channel::<(AuthCommand, oneshot::Sender<Response>)>(BUFFER_SIZE);

        let http_client = C::new(c_s);
        let db = D::new(c_r).await;

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
