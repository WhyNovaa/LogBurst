use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenvy::dotenv;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub db: String,
    pub pool_max_size: usize,
}

impl PostgresConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("POSTGRES_").from_env()
    }

    pub fn create_pool(self) -> Result<Pool, deadpool_postgres::BuildError> {
        let max_size = self.pool_max_size;
        let manager = Manager::from_config(
            self.into(),
            NoTls,
            ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            },
        );

        Pool::builder(manager).max_size(max_size).build()
    }
}

impl From<PostgresConfig> for tokio_postgres::Config {
    fn from(value: PostgresConfig) -> Self {
        let mut tokio_pg_cfg = Self::new();

        tokio_pg_cfg.host(value.host);
        tokio_pg_cfg.user(value.user);
        tokio_pg_cfg.password(value.password);
        tokio_pg_cfg.dbname(value.db);

        tokio_pg_cfg
    }
}
