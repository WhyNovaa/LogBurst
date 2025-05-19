use std::env;
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

pub struct ClickHouseClient {
    pub client: Client,
    pub log_command_receiver: LogCommandReceiver,
}

#[async_trait]
impl Start for ClickHouseClient {
    async fn start(mut self) {
        log::info!("Starting ClickHouseClient");

        loop {
            if let Some((command, response_sender)) = self.log_command_receiver.recv().await {
                log::info!("ClickHouse: {:?}", command);

                let response = self.handle_command(command).await;

                if let Err(response) = response_sender.send(response) {
                    log::error!("Couldn't send response: {response:?} to http client");
                }
            }
        }
    }
}

impl LogsRepository for ClickHouseClient {
    type Error = clickhouse::error::Error;

    fn new(log_command_receiver: LogCommandReceiver) -> Self {
        let user = env::var("CLICKHOUSE_USER").expect("CLICKHOUSE_USER not found in .env file");
        let password = env::var("CLICKHOUSE_PASSWORD").expect("CLICKHOUSE_PASSWORD not found in .env file");

        log::info!("Creating ClickHouseClient");
        let client = Client::default()
            .with_url("http://clickhouse-server:8123")
            .with_user(user)
            .with_password(password);

        Self {
            client,
            log_command_receiver
        }
    }

    async fn save_log(&self, log: &Log) -> Result<(), Self::Error> {
        let mut insert = self.client
            .insert("logs")?;

        insert.write(log)
            .await?;

        insert.end().await?;

        Ok(())
    }

    async fn get_logs(&self, service_name: &String) -> Result<Vec<Log>, Self::Error> {
        let req = "SELECT * FROM logs \
                        WHERE service = ? \
                        ORDER BY timestamp DESC";

        let logs = self.client
            .query(req)
            .bind(service_name)
            .fetch_all::<Log>()
            .await?;

        Ok(logs)
    }
}

impl ClickHouseClient {
    pub async fn handle_command(&self, command: LogCommand) -> Response {
        match command {
            LogCommand::SaveLog { log } => {
                match self.save_log(&log).await {
                    Ok(()) => {
                        let body = Json(json!({
                            "message": "log saved successfully"
                        }));

                        (StatusCode::OK, body).into_response()
                    }
                    Err(e) => {
                        log::error!("Error while saving log: {e}");

                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
                    }
                }
            }
            LogCommand::GetLogs { service_name} => {
                match self.get_logs(&service_name).await {
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
}