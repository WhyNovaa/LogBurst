use config::clickhouse::ClickhouseConfig;
use config::http::HttpConfig;
use config::jwt::JwtConfig;
use config::kafka::KafkaConfig;
use config::postgres::PostgresConfig;

pub struct Config {
    pub http: HttpConfig,
    pub postgres: PostgresConfig,
    pub jwt: JwtConfig,
    pub clickhouse: ClickhouseConfig,
    pub kafka: KafkaConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            http: HttpConfig::from_env()?,
            postgres: PostgresConfig::from_env()?,
            jwt: JwtConfig::from_env()?,
            clickhouse: ClickhouseConfig::from_env()?,
            kafka: KafkaConfig::from_env()?,
        })
    }
}
