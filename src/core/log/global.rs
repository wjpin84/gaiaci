use std::sync::{Arc, RwLock};
use crate::core::log::{LogLevel, LogSink};
use once_cell::sync::Lazy;

static LOGGER: Lazy<RwLock<Option<Arc<dyn LogSink>>>> = Lazy::new(|| RwLock::new(None));

pub fn set_logger<S: LogSink>(sink: S) {
    let mut logger = LOGGER.write().unwrap();
    *logger = Some(Arc::new(sink));
}

pub fn log(level: LogLevel, msg: &str) {
    if let Some(logger) = &*LOGGER.read().unwrap() {
        logger.log(level, msg);
    }
}

pub fn trace(msg: &str) { log(LogLevel::Trace, msg); }
pub fn debug(msg: &str) { log(LogLevel::Debug, msg); }
pub fn info(msg: &str)  { log(LogLevel::Info, msg); }
pub fn warn(msg: &str)  { log(LogLevel::Warn, msg); }
pub fn error(msg: &str) { log(LogLevel::Error, msg); }
