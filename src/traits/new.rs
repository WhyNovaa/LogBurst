use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::log::Log;

pub trait New {
    fn new(log_sender: mpsc::Sender<(Log, oneshot::Sender<Response>)>) -> Self;
}