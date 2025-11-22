use std::env;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::traits::logs_repository::LogsRepository;
use crate::traits::start::Start;
use clickhouse::Client;
use crate::models::app::LogCommandReceiver;
use crate::models::log::Log;
use crate::models::log_command::LogCommand;
use axum::Json;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::mpsc::error::SendError;
use tokio::time::interval;

const SAVE_INTERVAL: u64 = 500;
const BATCH_SAVE_AMOUNT: usize = 10000;

pub struct ClickHouseClient {
    pub client: Client,
}

/*#[async_trait]
impl Start for ClickHouseClient {
    fn start(mut self) {
        log::info!("Starting ClickHouseClient");

        let (tx, rx) = unbounded_channel::<Log>();

        tokio::spawn({
            let client = self.client.clone();
            async move {
                flush_loop(client, rx).await
            }
        });
    }
}*/

#[async_trait]
impl LogsRepository for ClickHouseClient {
    type Error = clickhouse::error::Error;

    fn new() -> Self {
        log::info!("Creating ClickHouseClient");

        let user = env::var("CLICKHOUSE_USER").expect("CLICKHOUSE_USER not found in .env file");
        let password = env::var("CLICKHOUSE_PASSWORD").expect("CLICKHOUSE_PASSWORD not found in .env file");

        let client = Client::default()
            .with_url("http://clickhouse-server:8123")
            .with_compression(clickhouse::Compression::Lz4)
            .with_user(user)
            .with_password(password);

        Self {
            client,
        }
    }

    async fn save_logs(&self, logs: &Vec<Log>) -> Result<(), Self::Error> {
        let mut insert = self.client
            .insert("logs")?;

        for log in logs {
            insert.write(log).await?;
        }

        insert.end().await?;

        Ok(())
    }

    async fn save_log(&self, log: &Log) -> Result<(), Self::Error> {
        let mut insert = self.client
            .insert("logs")?;

        insert.write(log).await?;

        insert.end().await?;

        Ok(())
    }

    async fn get_logs(&self, service: Option<String>, level: Option<String>) -> Result<Vec<Log>, Self::Error> {
        let mut bindings = Vec::new();
        let mut filters = Vec::new();

        let mut req = String::from("SELECT * FROM logs");

        if let Some(service) = service {
            bindings.push(service);
            filters.push("service = ?");
        }

        if let Some(level) = level {
            bindings.push(level);
            filters.push("level = ?");
        }

        if !filters.is_empty() {
            req.push_str(" WHERE ");
            req.push_str(&filters.join(" AND "));
        }

        let mut query = self.client.query(&req);

        for binding in bindings {
            query = query.bind(binding);
        }

        let logs = query.fetch_all::<Log>().await?;

        Ok(logs)
    }
}

/*impl ClickHouseClient {
    pub async fn handle_command(client: Client, tx: Arc<UnboundedSender<Log>>, command: LogCommand) -> Response {
        match command {
            LogCommand::SaveLog { log } => {
                if let Err(e) = tx.send(log) {
                    log::error!("Error while sending log: {e}");

                    return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
                }

                let body = Json(json!({
                    "message": "log saved successfully"
                }));

                (StatusCode::OK, body).into_response()
            }
            LogCommand::GetLogs { params} => {
                match ClickHouseClient::get_logs(&client, params.service, params.level).await {
                    Ok(logs) => {
                        (StatusCode::OK, Json(json!(logs))).into_response()
                    }
                    Err(e) => {
                        log::error!("Error while getting logs: {e}");

                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
                    }
                }
            }
        }
    }
}*/

/*pub async fn flush_loop(client: Client, mut rx: UnboundedReceiver<Log>) {
    let mut ticker = interval(Duration::from_millis(SAVE_INTERVAL));

    let mut buff = Vec::with_capacity(10000);

    loop {
/*        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    rx.recv_many(&mut buff, BATCH_SAVE_AMOUNT).await;
                    break;
                }

                Some(log) = rx.recv() => {
                    buff.push(log);
                    if buff.len() >= BATCH_SAVE_AMOUNT {
                        break;
                    }
                }

                else => {
                    break;
                }
            }*/
        ticker.tick().await;

        rx.recv_many(&mut buff, BATCH_SAVE_AMOUNT).await;

        match ClickHouseClient::save_logs(&client, &mut buff).await {
            Ok(_) => log::info!("Butch of logs were flushed successfully"),
            Err(e) => log::error!("Error while flushing butch of logs: {e}"),
        }

        buff.clear();
    }
}*/