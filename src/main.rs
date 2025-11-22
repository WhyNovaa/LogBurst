use crate::config::postgres::PostgresConfig;
use crate::db::pg::Postgres;
use tokio_postgres::types::ToSql;

mod db;
mod config;
mod rest;

#[tokio::main]
async fn main() {
    let cfg = PostgresConfig::from_env().unwrap();

    let pg = Postgres::connect(cfg).await;

    let params: Vec<&(dyn ToSql + Sync)> = Vec::new();

    let res = pg.client.execute_raw("SELECT 1 + 1", params).await.unwrap();

    dbg!("ABOBA");
}
