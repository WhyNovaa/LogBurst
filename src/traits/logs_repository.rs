use crate::models::app::LogCommandReceiver;
use crate::traits::start::Start;

pub trait LogsRepository: Start + Send + 'static {
    fn new(log_receiver: LogCommandReceiver) -> Self;
}