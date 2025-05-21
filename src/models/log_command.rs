use crate::models::http_client::get_logs_params::GetLogsParams;
use crate::models::log::Log;

#[derive(Debug)]
pub enum LogCommand {
    SaveLog {
        log: Log
    },
    GetLogs {
        params: GetLogsParams,
    }
}