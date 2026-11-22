mod get_user_by_username;

use crate::infrastructure::postgres::repository::PostgresUserRepository;

pub struct UserQueries {
    repo: PostgresUserRepository,
}

impl UserQueries {
    pub fn new(repo: PostgresUserRepository) -> Self {
        Self { repo }
    }
}
