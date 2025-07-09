use crate::core::log::{LogSink, LogLevel};

pub struct StdoutSink;

impl LogSink for StdoutSink {
    fn log(&self, level: LogLevel, msg: &str) {
        println!("[{}] {}", level, msg);
    }
}
