//! # Log Levels
//! 
//! This module defines the log severity levels used throughout the GaiaCI logging system.
//! Log levels follow standard conventions and are ordered from most verbose (Trace)
//! to least verbose (Error).

use std::fmt;

/// Enumeration of log severity levels.
/// 
/// Log levels are ordered by severity, with each level including all higher
/// severity levels. For example, if logging is set to `Info` level, it will
/// also log `Warn` and `Error` messages, but not `Debug` or `Trace`.
/// 
/// ## Level Descriptions
/// 
/// - **Trace** - Very detailed execution information, typically only of interest in diagnostics
/// - **Debug** - Detailed information useful for debugging during development
/// - **Info** - General information about normal program operation
/// - **Warn** - Warning messages about potentially harmful situations
/// - **Error** - Error messages about problems that require attention
/// 
/// ## Usage
/// 
/// ```rust
/// use gaiaci_core::log::LogLevel;
/// 
/// let level = LogLevel::Info;
/// println!("Current log level: {}", level); // Prints: "Current log level: INFO"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Most verbose level - fine-grained execution details
    Trace,
    /// Detailed debugging information for development
    Debug,
    /// General informational messages about normal operation
    Info,
    /// Warning messages about potential issues
    Warn,
    /// Error messages about problems requiring attention
    Error,
}

impl fmt::Display for LogLevel {
    /// Formats the log level as an uppercase string.
    /// 
    /// This implementation provides consistent string representation
    /// for log output formatting.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use gaiaci_core::log::LogLevel;
    /// assert_eq!(format!("{}", LogLevel::Info), "INFO");
    /// assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    /// Returns true if this log level is at least as severe as the specified level.
    /// 
    /// This method is useful for implementing log level filtering in custom sinks.
    /// 
    /// # Arguments
    /// 
    /// * `other` - The minimum log level to compare against
    /// 
    /// # Returns
    /// 
    /// `true` if this level is greater than or equal to `other` in severity
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use gaiaci_core::log::LogLevel;
    /// 
    /// assert!(LogLevel::Error.is_at_least(LogLevel::Warn));
    /// assert!(LogLevel::Info.is_at_least(LogLevel::Info));
    /// assert!(!LogLevel::Debug.is_at_least(LogLevel::Info));
    /// ```
    pub fn is_at_least(&self, other: LogLevel) -> bool {
        *self >= other
    }
    
    /// Returns the string representation in lowercase.
    /// 
    /// This can be useful for configuration files or APIs that expect
    /// lowercase log level names.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use gaiaci_core::log::LogLevel;
    /// 
    /// assert_eq!(LogLevel::Info.to_lowercase(), "info");
    /// assert_eq!(LogLevel::Error.to_lowercase(), "error");
    /// ```
    pub fn to_lowercase(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
    
    /// Parses a log level from a string (case-insensitive).
    /// 
    /// This method is useful for parsing log levels from configuration
    /// files or command-line arguments.
    /// 
    /// # Arguments
    /// 
    /// * `s` - The string to parse (case-insensitive)
    /// 
    /// # Returns
    /// 
    /// `Some(LogLevel)` if the string is a valid log level, `None` otherwise
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use gaiaci_core::log::LogLevel;
    /// 
    /// assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
    /// assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
    /// assert_eq!(LogLevel::from_str("Error"), Some(LogLevel::Error));
    /// assert_eq!(LogLevel::from_str("invalid"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<LogLevel> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Trace), "TRACE");
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Warn), "WARN");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
        
        // Test equality
        assert_eq!(LogLevel::Info, LogLevel::Info);
    }

    #[test]
    fn test_log_level_is_at_least() {
        assert!(LogLevel::Error.is_at_least(LogLevel::Warn));
        assert!(LogLevel::Error.is_at_least(LogLevel::Error));
        assert!(!LogLevel::Debug.is_at_least(LogLevel::Info));
        assert!(LogLevel::Info.is_at_least(LogLevel::Trace));
    }

    #[test]
    fn test_log_level_to_lowercase() {
        assert_eq!(LogLevel::Trace.to_lowercase(), "trace");
        assert_eq!(LogLevel::Debug.to_lowercase(), "debug");
        assert_eq!(LogLevel::Info.to_lowercase(), "info");
        assert_eq!(LogLevel::Warn.to_lowercase(), "warn");
        assert_eq!(LogLevel::Error.to_lowercase(), "error");
    }

    #[test]
    fn test_log_level_from_str() {
        // Test valid cases (case insensitive)
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("TRACE"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("Debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("warning"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("Error"), Some(LogLevel::Error));
        
        // Test invalid cases
        assert_eq!(LogLevel::from_str("invalid"), None);
        assert_eq!(LogLevel::from_str(""), None);
        assert_eq!(LogLevel::from_str("fatal"), None);
    }

    #[test]
    fn test_log_level_clone_copy() {
        let level = LogLevel::Info;
        let cloned = level.clone();
        let copied = level;
        
        assert_eq!(level, cloned);
        assert_eq!(level, copied);
        assert_eq!(cloned, copied);
    }
}
