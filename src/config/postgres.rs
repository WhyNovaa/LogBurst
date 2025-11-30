use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub db: String,
}

impl PostgresConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();
        envy::prefixed("POSTGRES_").from_env()
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