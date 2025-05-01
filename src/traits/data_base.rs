use crate::traits::auth_repository::AuthRepository;
use crate::traits::logs_repository::LogsRepository;

pub trait DataBase: AuthRepository + LogsRepository {

}