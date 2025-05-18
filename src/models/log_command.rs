use crate::models::log::Log;

#[derive(Debug)]
pub enum LogCommand {
    SaveLog {
        log: Log
    }
}