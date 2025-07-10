//! # Global Logger
//! 
//! This module provides the global logging interface for GaiaCI. It implements
//! a thread-safe global logger that can be configured with any `LogSink` implementation
//! and provides convenience functions for each log level.

use std::sync::{Arc, RwLock};
use crate::log::{LogLevel, LogSink};
use once_cell::sync::Lazy;

/// Global logger instance.
/// 
/// This static variable holds the global logger configuration. It uses `once_cell::Lazy`
/// for thread-safe lazy initialization and `RwLock` for safe concurrent access.
/// The logger is wrapped in `Option` to allow for uninitialized state.
static LOGGER: Lazy<RwLock<Option<Arc<dyn LogSink>>>> = Lazy::new(|| RwLock::new(None));

/// Sets the global logger to use the specified sink.
/// 
/// This function configures the global logging system to route all log messages
/// to the provided sink implementation. Once set, the sink will receive all
/// log messages generated through the convenience functions (`info`, `warn`, etc.).
/// 
/// # Arguments
/// 
/// * `sink` - The log sink implementation to use for all log output
/// 
/// # Thread Safety
/// 
/// This function is thread-safe and can be called from any thread. However,
/// it's recommended to set the logger once during application initialization
/// to avoid confusion about where log messages are being sent.
/// 
/// # Panics
/// 
/// This function will panic if the global logger's RwLock is poisoned,
/// which should only occur if another thread panicked while holding the lock.
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::{set_logger, info};
/// use gaiaci_core::log::sinks::stdout::StdoutSink;
/// 
/// // Configure logging to stdout
/// set_logger(StdoutSink);
/// 
/// // Now all log messages will go to stdout
/// info("Application initialized");
/// ```
pub fn set_logger<S: LogSink>(sink: S) {
    let mut logger = LOGGER.write().expect("Logger RwLock poisoned");
    *logger = Some(Arc::new(sink));
}

/// Logs a message at the specified level.
/// 
/// This is the core logging function that all convenience functions use.
/// If no logger has been configured via `set_logger`, the message will
/// be silently discarded.
/// 
/// # Arguments
/// 
/// * `level` - The severity level for this log message
/// * `msg` - The message content to log
/// 
/// # Error Handling
/// 
/// If the global logger's RwLock is poisoned, this function will silently
/// fail rather than panic, ensuring that logging failures don't crash
/// the application.
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::{log, LogLevel};
/// 
/// log(LogLevel::Info, "Custom log message");
/// log(LogLevel::Error, "Something went wrong");
/// ```
pub fn log(level: LogLevel, msg: &str) {
    if let Ok(logger_guard) = LOGGER.read() {
        if let Some(logger) = &*logger_guard {
            logger.log(level, msg);
        }
    }
    // Silently fail if RwLock is poisoned - logging shouldn't crash the app
}

/// Logs a message at TRACE level.
/// 
/// Trace messages provide the most detailed execution information and are
/// typically used for fine-grained debugging. These messages are usually
/// filtered out in production environments.
/// 
/// # Arguments
/// 
/// * `msg` - The trace message to log
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::trace;
/// 
/// trace("Entering function calculate_sum");
/// trace("Processing item 5 of 10");
/// ```
pub fn trace(msg: &str) { log(LogLevel::Trace, msg); }

/// Logs a message at DEBUG level.
/// 
/// Debug messages provide detailed information useful during development
/// and troubleshooting. These are typically enabled during development
/// but disabled in production.
/// 
/// # Arguments
/// 
/// * `msg` - The debug message to log
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::debug;
/// 
/// debug("Configuration loaded successfully");
/// debug("Database connection established");
/// ```
pub fn debug(msg: &str) { log(LogLevel::Debug, msg); }

/// Logs a message at INFO level.
/// 
/// Info messages provide general information about normal program operation.
/// These messages are typically shown in production to provide visibility
/// into application behavior.
/// 
/// # Arguments
/// 
/// * `msg` - The informational message to log
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::info;
/// 
/// info("Application starting");
/// info("Processing user request");
/// info("Task completed successfully");
/// ```
pub fn info(msg: &str)  { log(LogLevel::Info, msg); }

/// Logs a message at WARN level.
/// 
/// Warning messages indicate potentially harmful situations or deprecated
/// functionality. The application can continue running, but the situation
/// should be investigated.
/// 
/// # Arguments
/// 
/// * `msg` - The warning message to log
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::warn;
/// 
/// warn("Configuration file not found, using defaults");
/// warn("API rate limit approaching");
/// warn("Deprecated feature in use");
/// ```
pub fn warn(msg: &str)  { log(LogLevel::Warn, msg); }

/// Logs a message at ERROR level.
/// 
/// Error messages indicate serious problems that require attention. These
/// represent failures that may impact application functionality but don't
/// necessarily cause the application to terminate.
/// 
/// # Arguments
/// 
/// * `msg` - The error message to log
/// 
/// # Example
/// 
/// ```rust
/// use gaiaci_core::log::error;
/// 
/// error("Failed to connect to database");
/// error("Invalid configuration detected");
/// error("User authentication failed");
/// ```
pub fn error(msg: &str) { log(LogLevel::Error, msg); }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;

    /// Test sink that captures log messages for verification
    struct TestSink {
        messages: Arc<Mutex<VecDeque<(LogLevel, String)>>>,
    }

    impl TestSink {
        fn new() -> (Self, Arc<Mutex<VecDeque<(LogLevel, String)>>>) {
            let messages = Arc::new(Mutex::new(VecDeque::new()));
            (TestSink { messages: messages.clone() }, messages)
        }
    }

    impl LogSink for TestSink {
        fn log(&self, level: LogLevel, msg: &str) {
            self.messages.lock().unwrap().push_back((level, msg.to_string()));
        }
    }

    #[test]
    fn test_set_logger_and_log() {
        let (test_sink, messages) = TestSink::new();
        
        // Clear any existing messages before starting test
        messages.lock().unwrap().clear();
        
        set_logger(test_sink);
        
        // Use unique messages to identify our test's output
        let unique_id = std::process::id();
        let unique_info = format!("Test info message {}", unique_id);
        let unique_error = format!("Test error message {}", unique_id);
        
        // Test logging at different levels
        log(LogLevel::Info, &unique_info);
        log(LogLevel::Error, &unique_error);
        
        // Give a moment for logging to complete
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let captured = messages.lock().unwrap();
        
        // Find our messages in the captured log (other tests may have added messages)
        let info_found = captured.iter().any(|(level, msg)| 
            *level == LogLevel::Info && msg == &unique_info
        );
        let error_found = captured.iter().any(|(level, msg)| 
            *level == LogLevel::Error && msg == &unique_error
        );
        
        assert!(info_found, "Info message not found in log: {:?}", *captured);
        assert!(error_found, "Error message not found in log: {:?}", *captured);
    }

    #[test]
    fn test_convenience_functions() {
        let (test_sink, messages) = TestSink::new();
        
        // Clear any existing messages before starting test
        messages.lock().unwrap().clear();
        
        set_logger(test_sink);
        
        // Test all convenience functions
        trace("trace message");
        debug("debug message");
        info("info message");
        warn("warn message");
        error("error message");
        
        let captured = messages.lock().unwrap();
        assert_eq!(captured.len(), 5);
        assert_eq!(captured[0], (LogLevel::Trace, "trace message".to_string()));
        assert_eq!(captured[1], (LogLevel::Debug, "debug message".to_string()));
        assert_eq!(captured[2], (LogLevel::Info, "info message".to_string()));
        assert_eq!(captured[3], (LogLevel::Warn, "warn message".to_string()));
        assert_eq!(captured[4], (LogLevel::Error, "error message".to_string()));
    }

    #[test]
    fn test_log_without_logger_set() {
        // This test assumes we start without a logger set
        // In a real scenario, we might need to reset the global state
        
        // Try to log without setting a logger - should not panic
        log(LogLevel::Info, "This should be silently discarded");
        info("This should also be silently discarded");
        
        // Test passes if we reach this point without panicking
        assert!(true);
    }

    #[test]
    fn test_multiple_logger_replacements() {
        let (test_sink1, messages1) = TestSink::new();
        let (test_sink2, messages2) = TestSink::new();
        
        // Set first logger
        set_logger(test_sink1);
        info("Message to first logger");
        
        // Replace with second logger
        set_logger(test_sink2);
        info("Message to second logger");
        
        // Verify messages went to correct loggers
        assert_eq!(messages1.lock().unwrap().len(), 1);
        assert_eq!(messages2.lock().unwrap().len(), 1);
        
        assert_eq!(messages1.lock().unwrap()[0].1, "Message to first logger");
        assert_eq!(messages2.lock().unwrap()[0].1, "Message to second logger");
    }
}
