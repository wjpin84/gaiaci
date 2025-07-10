//! # Stdout Log Sink
//! 
//! This module provides a simple log sink implementation that outputs
//! all log messages to standard output (stdout) with basic formatting.

use crate::log::{LogSink, LogLevel};

/// A log sink that outputs messages to standard output.
/// 
/// `StdoutSink` is the simplest log sink implementation, suitable for
/// development, testing, and simple production deployments where console
/// output is appropriate.
/// 
/// ## Message Format
/// 
/// Messages are formatted as: `[LEVEL] message`
/// 
/// Where `LEVEL` is the uppercase string representation of the log level
/// (TRACE, DEBUG, INFO, WARN, ERROR).
/// 
/// ## Example Usage
/// 
/// ```rust
/// use gaiaci_core::log::{set_logger, info, warn};
/// use gaiaci_core::log::sinks::stdout::StdoutSink;
/// 
/// // Set up stdout logging
/// set_logger(StdoutSink);
/// 
/// // Log some messages
/// info("Application started");  // Output: [INFO] Application started
/// warn("Low disk space");       // Output: [WARN] Low disk space
/// ```
/// 
/// ## Performance Considerations
/// 
/// This sink uses `println!` macro which:
/// - Automatically flushes output after each message
/// - May block if stdout buffer is full
/// - Is suitable for moderate logging volumes
/// 
/// For high-throughput scenarios, consider implementing a buffered sink.
pub struct StdoutSink;

impl LogSink for StdoutSink {
    /// Outputs a log message to standard output.
    /// 
    /// This implementation formats the message with the log level and
    /// prints it to stdout using the `println!` macro. Each message
    /// is output on a separate line with automatic flushing.
    /// 
    /// # Arguments
    /// 
    /// * `level` - The severity level of the message
    /// * `msg` - The message content to output
    /// 
    /// # Example Output
    /// 
    /// ```text
    /// [INFO] Server starting on port 8080
    /// [WARN] Configuration file not found
    /// [ERROR] Database connection failed
    /// ```
    fn log(&self, level: LogLevel, msg: &str) {
        println!("[{}] {}", level, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_sink_basic() {
        let sink = StdoutSink;
        
        // This test verifies the sink compiles and can be called
        // In a real test environment, you might want to capture stdout
        sink.log(LogLevel::Info, "Test message");
        sink.log(LogLevel::Error, "Error message");
        
        // Test passes if no panic occurs
        assert!(true);
    }

    #[test]
    fn test_stdout_sink_implements_logsink() {
        let sink = StdoutSink;
        
        // Verify it implements LogSink trait
        fn accepts_log_sink<T: LogSink>(_: T) {}
        accepts_log_sink(sink);
    }

    #[test]
    fn test_stdout_sink_thread_safety() {
        // Verify Send + Sync traits
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<StdoutSink>();
        assert_sync::<StdoutSink>();
    }

    #[test]
    fn test_stdout_sink_with_different_levels() {
        let sink = StdoutSink;
        
        // Test with all log levels
        sink.log(LogLevel::Trace, "Trace message");
        sink.log(LogLevel::Debug, "Debug message");
        sink.log(LogLevel::Info, "Info message");
        sink.log(LogLevel::Warn, "Warn message");
        sink.log(LogLevel::Error, "Error message");
        
        // Test passes if no panic occurs
        assert!(true);
    }

    #[test]
    fn test_stdout_sink_with_special_characters() {
        let sink = StdoutSink;
        
        // Test with messages containing special characters
        sink.log(LogLevel::Info, "Message with unicode: 🦀 Rust!");
        sink.log(LogLevel::Info, "Message with newlines:\nLine 1\nLine 2");
        sink.log(LogLevel::Info, "Message with tabs:\tTabbed content");
        sink.log(LogLevel::Info, "Empty message: ");
        
        // Test passes if no panic occurs
        assert!(true);
    }
}
