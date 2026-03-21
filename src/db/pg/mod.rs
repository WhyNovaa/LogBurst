use crate::config::postgres::PostgresConfig;
use crate::db::pg::structs::User;
use deadpool_postgres::{GenericClient, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod};
use futures::TryFutureExt;
use tokio_postgres::{Error, NoTls};

pub mod structs;

pub struct Postgres {
    pool: deadpool_postgres::Pool,
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

        Self { pool }
    }

    pub async fn client(&self) -> Result<deadpool_postgres::Client, deadpool_postgres::PoolError> {
        self.pool.get().await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, anyhow::Error> {
        let req = "SELECT * FROM users WHERE username = $1";

        let res = self.client().await?.query_opt(req, &[&username]).await?;

        Ok(res.map(User::from))
    }
}
