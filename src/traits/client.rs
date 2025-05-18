use crate::models::app::{AuthCommandSender, LogCommandSender};
use crate::traits::start::Start;

pub trait Client: Start + Send + 'static {
    fn new(auth_command_sender: AuthCommandSender, log_command_sender: LogCommandSender) -> Self;
}