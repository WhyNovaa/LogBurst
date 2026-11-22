use crate::log_proto::LogLevel;

pub mod log_proto {
    tonic::include_proto!("log_collector");
}

pub mod kafka {
    tonic::include_proto!("logburst.kafka.v1");
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
