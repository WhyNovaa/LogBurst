use crate::config::postgres::PostgresConfig;
use crate::db::pg::structs::User;
use deadpool_postgres::{GenericClient, Manager, ManagerConfig, Pool, RecyclingMethod};
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

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let req = "SELECT * FROM users WHERE username = $1";

        let res = self.client.query_opt(req, &[&username]).await?;

        Ok(res.map(User::from))
    }
}