//! # Configurable Stdout Log Sink
//! 
//! This module provides a configurable stdout sink that supports custom
//! log formatters for flexible output formatting.

use crate::core::log::{LogSink, LogLevel, LogFormatter, LogContext, SimpleFormatter};
use std::sync::Arc;

/// A configurable log sink that outputs messages to standard output.
/// 
/// `ConfigurableStdoutSink` extends the basic stdout functionality by
/// accepting custom formatters, allowing users to control the exact
/// format of log messages output to the console.
/// 
/// ## Example Usage
/// 
/// ```rust
/// use gaiaci::core::log::{set_logger, info};
/// use gaiaci::core::log::sinks::configurable_stdout::ConfigurableStdoutSink;
/// use gaiaci::core::log::formatters::TemplateFormatter;
/// 
/// // Create a sink with custom formatting
/// let formatter = TemplateFormatter::new("[{timestamp:%H:%M:%S}] [{level}] {message}");
/// let sink = ConfigurableStdoutSink::new(formatter);
/// set_logger(sink);
/// 
/// info("Application started");  // Output: [14:30:15] [INFO] Application started
/// ```
pub struct ConfigurableStdoutSink {
    formatter: Arc<dyn LogFormatter>,
}

impl ConfigurableStdoutSink {
    /// Creates a new ConfigurableStdoutSink with the specified formatter.
    /// 
    /// # Arguments
    /// 
    /// * `formatter` - The formatter to use for log messages
    pub fn new<F: LogFormatter>(formatter: F) -> Self {
        Self {
            formatter: Arc::new(formatter),
        }
    }

    /// Creates a new sink with the simple default formatter.
    /// 
    /// This is equivalent to the basic StdoutSink but uses the
    /// configurable architecture.
    pub fn simple() -> Self {
        Self::new(SimpleFormatter)
    }

    /// Creates a new sink with a CI/CD friendly format.
    /// 
    /// Format: `[timestamp] [user] [level] message`
    pub fn ci_format() -> Self {
        use crate::core::log::formatters::TemplateFormatter;
        Self::new(TemplateFormatter::ci_format())
    }

    /// Creates a new sink with JSON output format.
    pub fn json_format() -> Self {
        use crate::core::log::formatters::TemplateFormatter;
        Self::new(TemplateFormatter::json_format())
    }

    /// Creates a new sink optimized for development/debugging.
    /// 
    /// Format: `[HH:MM:SS] [thread] [LEVEL] message`
    pub fn dev_format() -> Self {
        use crate::core::log::formatters::TemplateFormatter;
        Self::new(TemplateFormatter::dev_format())
    }

    /// Creates a new sink with a custom template.
    /// 
    /// # Arguments
    /// 
    /// * `template` - The format template string
    pub fn with_template<S: Into<String>>(template: S) -> Self {
        use crate::core::log::formatters::TemplateFormatter;
        Self::new(TemplateFormatter::new(template))
    }
}

impl LogSink for ConfigurableStdoutSink {
    /// Outputs a formatted log message to standard output.
    /// 
    /// This implementation creates a LogContext with the provided level
    /// and message, formats it using the configured formatter, and
    /// prints the result to stdout.
    /// 
    /// # Arguments
    /// 
    /// * `level` - The log level
    /// * `msg` - The log message
    fn log(&self, level: LogLevel, msg: &str) {
        let context = LogContext::new(level, msg.to_string());
        let formatted = self.formatter.format(&context);
        println!("{}", formatted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurable_stdout_sink_simple() {
        let sink = ConfigurableStdoutSink::simple();
        // Test that it can be called without panicking
        sink.log(LogLevel::Info, "test message");
    }

    #[test]
    fn test_configurable_stdout_sink_with_template() {
        let sink = ConfigurableStdoutSink::with_template("[{level}] {message}");
        // Test that it can be called without panicking
        sink.log(LogLevel::Warn, "warning message");
    }

    #[test]
    fn test_configurable_stdout_sink_ci_format() {
        let sink = ConfigurableStdoutSink::ci_format();
        // Test that it can be called without panicking
        sink.log(LogLevel::Error, "error message");
    }
}
