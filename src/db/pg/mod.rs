use crate::config::postgres::PostgresConfig;
use crate::db::pg::structs::User;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Error, NoTls};

pub mod structs;

pub struct Postgres {
    client: deadpool_postgres::Client,
}

impl Postgres {
    pub async fn connect(cfg: PostgresConfig) -> Self {
        let mgr = Manager::from_config(
            cfg.into(),
            NoTls,
            ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            },
        );

        let pool = Pool::builder(mgr).max_size(16).build().unwrap();

        let client = pool.get().await.unwrap();

        Self {
            client
        }
    }

    pub async fn login(&self, user: User) -> Result<bool, Error> {
        let req = "SELECT 1 FROM users WHERE username = $1 AND hashed_password = $2";

        let res = self.client.query_opt(req, &[&user.username, &user.hashed_password]).await?;

        Ok(res.is_some())
    }
}