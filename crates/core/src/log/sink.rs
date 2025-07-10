//! # Log Sink Trait
//! 
//! This module defines the `LogSink` trait, which provides the interface for
//! implementing custom log output destinations in the GaiaCI logging system.

use crate::log::levels::LogLevel;

/// Trait for implementing custom log output destinations.
/// 
/// The `LogSink` trait defines the interface that all log output implementations
/// must follow. Sinks are responsible for formatting and outputting log messages
/// to their respective destinations (stdout, files, network, etc.).
/// 
/// ## Thread Safety Requirements
/// 
/// All `LogSink` implementations must be:
/// - `Send` - Safe to transfer between threads
/// - `Sync` - Safe to share references between threads
/// - `'static` - Contains no borrowed data with shorter lifetimes
/// 
/// These requirements ensure that sinks can be used safely in the global
/// logging system across multiple threads.
/// 
/// ## Implementation Example
/// 
/// ```rust
/// use gaiaci_core::log::{LogSink, LogLevel};
/// 
/// struct FileSink {
///     file_path: String,
/// }
/// 
/// impl LogSink for FileSink {
///     fn log(&self, level: LogLevel, msg: &str) {
///         // Write to file with timestamp and level
///         let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
///         let formatted = format!("[{}] [{}] {}\n", timestamp, level, msg);
///         std::fs::write(&self.file_path, formatted).unwrap();
///     }
/// }
/// ```
pub trait LogSink: Send + Sync + 'static {
    /// Processes and outputs a log message.
    /// 
    /// This method is called by the logging system for each log message
    /// that should be output. Implementations should handle the formatting,
    /// buffering, and actual output of the message as appropriate for their
    /// destination.
    /// 
    /// # Arguments
    /// 
    /// * `level` - The severity level of the log message
    /// * `msg` - The log message content
    /// 
    /// # Implementation Notes
    /// 
    /// - This method should be non-blocking when possible
    /// - Error handling should be done internally (avoid panicking)
    /// - Consider buffering for performance in high-throughput scenarios
    /// - Thread safety is handled at the global logger level
    fn log(&self, level: LogLevel, msg: &str);
}
