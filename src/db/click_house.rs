use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::traits::logs_repository::LogsRepository;
use crate::traits::start::Start;
use clickhouse::Client;
use crate::models::app::{LogCommandReceiver, LogCommandSender};
use crate::models::log::Log;
use crate::models::log_command::LogCommand;

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
    fn new(log_command_receiver: LogCommandReceiver) -> Self {
        log::info!("Creating ClickHouseClient");
        let client = Client::default()
            .with_url("http://clickhouse-server:8123")
            .with_user("admin")
            .with_password("admin");

        Self {
            client,
            log_command_receiver
        }
    }
}

impl ClickHouseClient {
    pub async fn handle_command(&self, command: LogCommand) -> Response {
        match command {
            LogCommand::SaveLog { log } => {
                (StatusCode::OK, "Todo!").into_response()
            }
        }
    }
}