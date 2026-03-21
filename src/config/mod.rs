use crate::config::clickhouse::ClickhouseConfig;
use crate::config::grpc::GrpcConfig;
use crate::config::postgres::PostgresConfig;
use crate::config::rest::ServerConfig;

pub mod clickhouse;
pub mod grpc;
pub mod postgres;
pub mod rest;

#[derive(Debug, Clone)]
pub struct Config {
    pub rest_cfg: ServerConfig,
    pub pg_cfg: PostgresConfig,
    pub clickhouse_cfg: ClickhouseConfig,
    pub grpc_config: GrpcConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            rest_cfg: ServerConfig::from_env()?,
            pg_cfg: PostgresConfig::from_env()?,
            clickhouse_cfg: ClickhouseConfig::from_env()?,
            grpc_config: GrpcConfig::from_env()?,
        })
    }
}
