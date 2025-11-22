use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use crate::handlers::errors::AuthError;
use crate::models::app::LogCommandSender;
use crate::models::http_client::get_logs_params::GetLogsParams;
use crate::models::log::Log;
use crate::models::log_command::LogCommand;
use crate::traits::logs_repository::LogsRepository;

pub async fn save_log<L: LogsRepository> (
    State(logs_db): State<Arc<L>>,
    Json(log): Json<Log>,
) -> Response {
    log::info!("Save log endpoint: {:?}", log);

    if let Err(e) = logs_db.save_log(&log).await {
        let body = Json(json!({
                    "message": "log wasn't saved successfully"
                }));

        return (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }

    let body = Json(json!({
                    "message": "log saved successfully"
                }));

    (StatusCode::OK, body).into_response()
}

pub async fn get_logs<L: LogsRepository>(
    State(logs_db): State<Arc<L>>,
/*    Query(params): Query<GetLogsParams>,*/
) -> Response {
/*    log::info!("Get_logs endpoint: {:?}", params);*/

/*    match logs_db.get_logs(params.service, params.level).await  {
        Ok(logs) => {
            (StatusCode::OK, Json(json!(logs))).into_response()
        }
        Err(e) => {
            log::error!("Error while getting logs: {e}");

            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }*/
    StatusCode::OK.into_response()
}