use crate::traits::start::Start;

pub trait LogsRepository: Start + Send + 'static {
    fn new() -> Self;
}