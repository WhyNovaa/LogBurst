use axum::response::Response;
use tokio::sync::{mpsc, oneshot};
use crate::models::app::{AuthCommandSender, LogSender};
use crate::models::log::Log;
use crate::traits::start::Start;

pub trait Client: Start {
    fn new(auth_command_sender: AuthCommandSender, log_sender: LogSender) -> Self;
}