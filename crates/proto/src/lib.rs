use crate::log_proto::LogLevel;

pub mod log_proto {
    tonic::include_proto!("log_collector");
}

impl From<LogLevel> for String {
    fn from(value: LogLevel) -> Self {
        let res = match value {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };

        res.to_string()
    }
}
