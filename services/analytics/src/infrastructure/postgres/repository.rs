use crate::infrastructure::error::DbError;
use crate::infrastructure::postgres::rows::user::User;

pub struct PostgresUserRepository {
    pool: deadpool_postgres::Pool,
}

impl PostgresUserRepository {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        Self { pool }
    }

    async fn client(&self) -> Result<deadpool_postgres::Client, deadpool_postgres::PoolError> {
        self.pool.get().await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DbError> {
        const QUERY: &str = "SELECT * FROM users WHERE username = $1";

        let res = self.client().await?.query_opt(QUERY, &[&username]).await?;

        Ok(res.map(User::from))
    }
}
