// core/log/sink.rs
use crate::core::log::levels::LogLevel;

pub trait LogSink: Send + Sync + 'static {
    fn log(&self, level: LogLevel, msg: &str);
}
