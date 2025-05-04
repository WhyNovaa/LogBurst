use crate::traits::auth_repository::AuthRepository;
use crate::traits::data_base::DataBase;
use crate::traits::logs_repository::LogsRepository;

/*pub struct DBWorker<A, R>
where A: AuthRepository,
    R: LogsRepository,
{
    auth: A,
    logs: R
}

impl<A: AuthRepository, R: LogsRepository> AuthRepository for DBWorker<A, R> {
    async fn new() -> Self {
        todo!()
    }
}

impl<A, R> LogsRepository for DBWorker<A, R> {}

impl<A, R> DataBase for DBWorker<A, R> {}*/