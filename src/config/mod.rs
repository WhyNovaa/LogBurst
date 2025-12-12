use crate::config::clickhouse::ClickhouseConfig;
use crate::config::postgres::PostgresConfig;
use crate::config::rest::RestConfig;

pub mod postgres;
pub mod rest;
pub mod clickhouse;

#[derive(Debug, Clone)]
pub struct Config {
    pub rest_cfg: RestConfig,
    pub pg_cfg: PostgresConfig,
    pub clickhouse_cfg: ClickhouseConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            rest_cfg: RestConfig::from_env()?,
            pg_cfg: PostgresConfig::from_env()?,
            clickhouse_cfg: ClickhouseConfig::from_env()?,
        })
    }
}