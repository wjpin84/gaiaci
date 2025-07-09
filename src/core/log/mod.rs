pub mod levels;
pub mod sink;
pub mod sinks;
pub mod global;

pub use levels::LogLevel;
pub use sink::LogSink;
pub use global::{set_logger, log, info, warn, error, debug, trace};
