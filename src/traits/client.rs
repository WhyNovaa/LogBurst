use crate::models::app::AuthCommandSender;
use crate::traits::start::Start;

pub trait Client: Start + Send + 'static {
    fn new(auth_command_sender: AuthCommandSender) -> Self;
}