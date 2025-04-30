use crate::traits::auth::Auth;
use crate::traits::logs_repository::LogsRepository;

pub struct DBWorker<A, R>
where A: Auth,
    R: LogsRepository,
{
    auth: A,
    logs: R
}