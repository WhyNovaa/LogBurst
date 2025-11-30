use crate::config::Config;
use crate::db::pg::Postgres;
use tokio_util::sync::CancellationToken;

pub struct Server {
    pub auth_pool: Postgres,
    pub token: CancellationToken,
}

impl Server {
    pub async fn new(cfg: Config) -> Self {
        let auth_pool = Postgres::connect(cfg.pg_cfg).await;

        let token = CancellationToken::default();

        Self {
            auth_pool,
            token,
        }
    }
}