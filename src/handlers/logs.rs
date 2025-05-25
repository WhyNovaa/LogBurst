use axum::extract::{Query, State};
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::handlers::errors::AuthError;
use crate::models::app::LogCommandSender;
use crate::models::http_client::get_logs_params::GetLogsParams;
use crate::models::log::Log;
use crate::models::log_command::LogCommand;

pub async fn save_log(
    State(log_command_sender): State<LogCommandSender>,
    Json(log): Json<Log>,
) -> Response {
    log::info!("Save log endpoint: {:?}", log);

    let command = LogCommand::SaveLog { log };

    send_command(&log_command_sender, command).await
}

pub async fn get_logs(
    State(log_command_sender): State<LogCommandSender>,
    Query(params): Query<GetLogsParams>,
) -> Response {
    log::info!("Get_logs endpoint: {:?}", params);

    let command = LogCommand::GetLogs {
        params,
    };

    send_command(&log_command_sender, command).await
}

async fn send_command(command_sender: &LogCommandSender, command: LogCommand) -> Response {
    let (one_s, one_r) = tokio::sync::oneshot::channel::<Response>();

    if let Err(e) = command_sender.send((command, one_s)).await {
        log::error!("Failed to send command: {}", e);
        return AuthError::InternalServerError.into_response()
    }

    one_r.await.unwrap_or_else(|e| {
        log::error!("oneshot receive failed: {}", e);
        AuthError::InternalServerError.into_response()
    })
}