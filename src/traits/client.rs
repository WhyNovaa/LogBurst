use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::log::Log;
use crate::traits::start::Start;

pub trait Client: Start {
    fn new(log_sender: mpsc::Sender<(Log, oneshot::Sender<Response>)>) -> Self;
}