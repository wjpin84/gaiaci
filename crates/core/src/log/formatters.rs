//! # Log Formatter Trait and Implementations
//! 
//! This module provides flexible log message formatting capabilities for the GaiaCI
//! logging system. It supports template-based formatting with various placeholders
//! for timestamps, log levels, user information, and custom fields.

use crate::log::LogLevel;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Context information available for log formatting.
/// 
/// This struct contains all the contextual information that can be used
/// when formatting log messages, including timestamps, user information,
/// process details, and custom fields.
#[derive(Debug, Clone)]
pub struct LogContext {
    /// The timestamp when the log message was created
    pub timestamp: DateTime<Utc>,
    /// The log level (TRACE, DEBUG, INFO, WARN, ERROR)
    pub level: LogLevel,
    /// The actual log message content
    pub message: String,
    /// Optional user identifier (e.g., CI user, system user)
    pub user: Option<String>,
    /// Optional process identifier
    pub process_id: Option<u32>,
    /// Optional thread identifier
    pub thread_id: Option<String>,
    /// Optional source location (file:line)
    pub source: Option<String>,
    /// Custom key-value pairs for additional context
    pub custom_fields: HashMap<String, String>,
}

impl LogContext {
    /// Creates a new LogContext with the current timestamp and provided level/message.
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
            user: None,
            process_id: Some(std::process::id()),
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            source: None,
            custom_fields: HashMap::new(),
        }
    }

    /// Sets the user field.
    pub fn with_user<S: Into<String>>(mut self, user: S) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets the source location.
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Adds a custom field.
    pub fn with_field<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.custom_fields.insert(key.into(), value.into());
        self
    }
}

/// Trait for implementing custom log message formatters.
/// 
/// LogFormatter implementations define how log messages are formatted before
/// being output by log sinks. This allows for flexible formatting strategies
/// including simple text, JSON, structured logging formats, etc.
pub trait LogFormatter: Send + Sync + 'static {
    /// Formats a log message using the provided context.
    /// 
    /// # Arguments
    /// 
    /// * `context` - The log context containing all available information
    /// 
    /// # Returns
    /// 
    /// A formatted string ready for output
    fn format(&self, context: &LogContext) -> String;
}

/// Simple formatter that outputs messages in a basic format.
/// 
/// Format: `[LEVEL] message`
pub struct SimpleFormatter;

impl LogFormatter for SimpleFormatter {
    fn format(&self, context: &LogContext) -> String {
        format!("[{}] {}", context.level, context.message)
    }
}

/// Detailed formatter with timestamp and process information.
/// 
/// Format: `[timestamp] [PID] [LEVEL] message`
pub struct DetailedFormatter;

impl LogFormatter for DetailedFormatter {
    fn format(&self, context: &LogContext) -> String {
        let timestamp = context.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let pid = context.process_id.unwrap_or(0);
        format!("[{}] [{}] [{}] {}", timestamp, pid, context.level, context.message)
    }
}

/// Template-based formatter that supports placeholder substitution.
/// 
/// Supported placeholders:
/// - `{timestamp}` - Full timestamp in ISO format
/// - `{timestamp:format}` - Timestamp with custom format (e.g., `{timestamp:%Y-%m-%d %H:%M:%S}`)
/// - `{level}` - Log level in uppercase
/// - `{level:lower}` - Log level in lowercase
/// - `{message}` - The log message content
/// - `{user}` - User identifier (if available)
/// - `{pid}` - Process ID
/// - `{thread}` - Thread ID
/// - `{source}` - Source location (if available)
/// - `{field:name}` - Custom field value
/// 
/// ## Example Templates
/// 
/// ```text
/// // Simple format
/// "[{level}] {message}"
/// 
/// // Detailed format with timestamp
/// "{timestamp:%Y-%m-%d %H:%M:%S} [{level}] {message}"
/// 
/// // JSON-like format
/// "{\"timestamp\":\"{timestamp}\",\"level\":\"{level:lower}\",\"message\":\"{message}\"}"
/// 
/// // CI-focused format
/// "[{timestamp:%H:%M:%S}] [{user}] [{level}] {message}"
/// ```
pub struct TemplateFormatter {
    template: String,
}

impl TemplateFormatter {
    /// Creates a new TemplateFormatter with the specified template.
    /// 
    /// # Arguments
    /// 
    /// * `template` - The format template string with placeholders
    pub fn new<S: Into<String>>(template: S) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Creates a formatter with a standard CI/CD friendly format.
    /// 
    /// Format: `[timestamp] [user] [level] message`
    pub fn ci_format() -> Self {
        Self::new("[{timestamp:%Y-%m-%d %H:%M:%S}] [{user}] [{level}] {message}")
    }

    /// Creates a formatter with JSON output format.
    pub fn json_format() -> Self {
        Self::new(r#"{{"timestamp":"{timestamp}","level":"{level:lower}","message":"{message}","user":"{user}","pid":{pid}}}"#)
    }

    /// Creates a formatter for development/debugging.
    /// 
    /// Format: `[HH:MM:SS] [thread] [LEVEL] message`
    pub fn dev_format() -> Self {
        Self::new("[{timestamp:%H:%M:%S}] [{thread}] [{level}] {message}")
    }
}

impl LogFormatter for TemplateFormatter {
    fn format(&self, context: &LogContext) -> String {
        let mut result = self.template.clone();

        // Replace timestamp placeholders
        if result.contains("{timestamp}") {
            result = result.replace("{timestamp}", &context.timestamp.to_rfc3339());
        }

        // Handle custom timestamp formats
        if let Some(start) = result.find("{timestamp:") {
            if let Some(end) = result[start..].find('}') {
                let placeholder = &result[start..start + end + 1];
                let format_str = &placeholder[12..placeholder.len() - 1]; // Extract format between : and }
                let formatted_time = context.timestamp.format(format_str).to_string();
                result = result.replace(placeholder, &formatted_time);
            }
        }

        // Replace level placeholders
        result = result.replace("{level}", &context.level.to_string());
        result = result.replace("{level:lower}", &context.level.to_lowercase());

        // Replace message
        result = result.replace("{message}", &context.message);

        // Replace user
        result = result.replace("{user}", context.user.as_deref().unwrap_or("unknown"));

        // Replace process ID
        result = result.replace("{pid}", &context.process_id.unwrap_or(0).to_string());

        // Replace thread ID
        result = result.replace("{thread}", context.thread_id.as_deref().unwrap_or("unknown"));

        // Replace source
        result = result.replace("{source}", context.source.as_deref().unwrap_or(""));

        // Replace custom fields
        for (key, value) in &context.custom_fields {
            let placeholder = format!("{{field:{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_formatter() {
        let formatter = SimpleFormatter;
        let context = LogContext::new(LogLevel::Info, "test message".to_string());
        let result = formatter.format(&context);
        assert_eq!(result, "[INFO] test message");
    }

    #[test]
    fn test_detailed_formatter() {
        let formatter = DetailedFormatter;
        let context = LogContext::new(LogLevel::Warn, "warning message".to_string());
        let result = formatter.format(&context);
        assert!(result.contains("[WARN] warning message"));
        assert!(result.contains("]"));  // Should contain timestamp and PID
    }

    #[test]
    fn test_template_formatter_basic() {
        let formatter = TemplateFormatter::new("[{level}] {message}");
        let context = LogContext::new(LogLevel::Error, "error occurred".to_string());
        let result = formatter.format(&context);
        assert_eq!(result, "[ERROR] error occurred");
    }

    #[test]
    fn test_template_formatter_with_user() {
        let formatter = TemplateFormatter::new("[{user}] [{level}] {message}");
        let context = LogContext::new(LogLevel::Info, "deploy started".to_string())
            .with_user("ci-bot");
        let result = formatter.format(&context);
        assert_eq!(result, "[ci-bot] [INFO] deploy started");
    }

    #[test]
    fn test_template_formatter_custom_fields() {
        let formatter = TemplateFormatter::new("{level}: {message} (build: {field:build_id})");
        let context = LogContext::new(LogLevel::Info, "build completed".to_string())
            .with_field("build_id", "12345");
        let result = formatter.format(&context);
        assert_eq!(result, "INFO: build completed (build: 12345)");
    }

    #[test]
    fn test_json_formatter() {
        let formatter = TemplateFormatter::json_format();
        let context = LogContext::new(LogLevel::Debug, "debug info".to_string())
            .with_user("developer");
        let result = formatter.format(&context);
        assert!(result.contains("\"level\":\"debug\""));
        assert!(result.contains("\"message\":\"debug info\""));
        assert!(result.contains("\"user\":\"developer\""));
    }
}
