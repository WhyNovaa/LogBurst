use axum::response::Response;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use crate::models::log::Log;
use crate::traits::auth_repository::AuthRepository;
use crate::traits::logs_repository::LogsRepository;

pub trait DataBase: AuthRepository + LogsRepository {
    fn new(receiver: Receiver<(Log, Sender<Response>)>) -> Self;
}