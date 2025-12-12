use crate::config::Config;
use crate::db::clickhouse::ClickHouse;
use crate::db::pg::Postgres;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct Server {
    pub auth_pool: Postgres,
    pub logs_db: ClickHouse,
    pub token: CancellationToken,
}

impl Server {
    pub async fn new(cfg: Arc<Config>) -> Self {
        let auth_pool = Postgres::connect(cfg.pg_cfg.clone()).await;

        let logs_db = ClickHouse::connect(cfg.clickhouse_cfg.clone()).await;

        let token = CancellationToken::default();

        Self {
            auth_pool,
            logs_db,
            token,
        }
    }
}