use crate::models::app::{AuthCommandSender, LogCommandSender};
use crate::traits::logs_repository::LogsRepository;
use crate::traits::start::Start;

pub trait Client<L: LogsRepository>: Start + Send + Sync + 'static {
    fn new(auth_command_sender: AuthCommandSender) -> Self;
}