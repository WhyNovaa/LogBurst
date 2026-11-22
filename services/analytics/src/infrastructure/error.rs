#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("ClickHouse query failed: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),

    #[error("Postgres query failed: {0}")]
    Postgres(#[from] tokio_postgres::error::Error),

    #[error("Pool error: {0}")]
    PostgresPool(#[from] deadpool_postgres::PoolError),
}
