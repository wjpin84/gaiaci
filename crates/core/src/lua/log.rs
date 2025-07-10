//! # Log Module
//! 
//! This module provides logging functions for Lua scripts in GaiaCI pipelines.
//! It integrates with GaiaCI's centralized logging system to provide structured
//! logging at various levels, enabling proper monitoring and debugging of CI processes.
//! 
//! ## Available Functions
//! 
//! ### Basic Logging Functions
//! - `trace(message)` - Log a trace-level message (most verbose)
//! - `debug(message)` - Log a debug-level message (verbose, for development)
//! - `info(message)` - Log an informational message (normal operation)
//! - `warn(message)` - Log a warning message (potential issues)
//! - `error(message)` - Log an error message (problems requiring attention)
//! 
//! ### Configuration Functions
//! - `set_format(template)` - Set a custom log format template
//! - `set_ci_format()` - Use CI/CD friendly format
//! - `set_json_format()` - Use JSON output format
//! - `set_dev_format()` - Use development/debugging format
//! - `set_simple_format()` - Use simple format (default)
//! 
//! ## Log Levels
//! 
//! The log levels follow standard logging conventions, from most to least verbose:
//! 1. **Trace** - Very detailed execution information
//! 2. **Debug** - Detailed information for debugging
//! 3. **Info** - General information about normal operation
//! 4. **Warn** - Warning messages about potential issues
//! 5. **Error** - Error messages about problems that need attention
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Basic logging at different levels
//! log.info("Starting CI pipeline")
//! log.debug("Processing configuration file")
//! log.warn("Deprecated feature in use")
//! log.error("Failed to connect to database")
//! log.trace("Variable x = " .. tostring(x))
//! 
//! -- Configure log formatting
//! log.set_ci_format()  -- Use CI/CD friendly format
//! log.info("Build started")  -- [2023-07-10 14:30:15] [unknown] [INFO] Build started
//! 
//! log.set_json_format()  -- Switch to JSON format
//! log.info("Deploy completed")  -- {"timestamp":"2023-07-10T14:30:16Z","level":"info","message":"Deploy completed",...}
//! 
//! -- Custom format template
//! log.set_format("[{timestamp:%H:%M:%S}] {level}: {message}")
//! log.warn("Custom format warning")  -- [14:30:17] WARN: Custom format warning
//! 
//! -- Conditional logging
//! if some_condition then
//!     log.info("Condition met, proceeding with operation")
//! else
//!     log.warn("Condition not met, using fallback")
//! end
//! ```

use mlua::{Lua, Result as LuaResult, Table};
use super::GaiaModule;
use crate::log;
use crate::log::sinks::configurable_stdout::ConfigurableStdoutSink;

/// The Log module provides structured logging functions for Lua scripts.
/// 
/// This struct implements the GaiaModule trait to expose logging functions
/// to Lua environments, integrating with GaiaCI's centralized logging system
/// for consistent log formatting and routing.
pub struct Log;

impl GaiaModule for Log {
    /// Creates a new Log module table with all logging functions.
    /// 
    /// This method registers all available logging functions with the Lua environment,
    /// making them accessible under the `log` namespace. Each function corresponds
    /// to a different log level in the GaiaCI logging system.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context to create the module in
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing all logging functions
    fn create(lua: &Lua) -> LuaResult<Table> {
        let log_table = lua.create_table()?;
        
        // Basic logging functions
        log_table.set("info", lua.create_function(log_info)?)?;
        log_table.set("warn", lua.create_function(log_warn)?)?;
        log_table.set("error", lua.create_function(log_error)?)?;
        log_table.set("debug", lua.create_function(log_debug)?)?;
        log_table.set("trace", lua.create_function(log_trace)?)?;
        
        // Formatting configuration functions
        log_table.set("set_format", lua.create_function(set_format)?)?;
        log_table.set("set_ci_format", lua.create_function(set_ci_format)?)?;
        log_table.set("set_json_format", lua.create_function(set_json_format)?)?;
        log_table.set("set_dev_format", lua.create_function(set_dev_format)?)?;
        log_table.set("set_simple_format", lua.create_function(set_simple_format)?)?;
        
        Ok(log_table)
    }
}

/// Logs an informational message.
/// 
/// This function logs a message at the INFO level, which is used for general
/// information about normal program operation. Info messages are typically
/// displayed in normal operation and help track the progress of CI pipelines.
/// 
/// # Arguments
/// 
/// * `msg` - The message to log
/// 
/// # Example
/// 
/// ```lua
/// log.info("Starting build process")
/// log.info("Tests completed successfully")
/// ```
fn log_info(_: &Lua, msg: String) -> LuaResult<()> {
    log::info(&msg);
    Ok(())
}

/// Logs a debug message.
/// 
/// This function logs a message at the DEBUG level, which is used for detailed
/// information that is typically only of interest when diagnosing problems.
/// Debug messages are usually filtered out in production but are valuable
/// during development and troubleshooting.
/// 
/// # Arguments
/// 
/// * `msg` - The message to log
/// 
/// # Example
/// 
/// ```lua
/// log.debug("Processing configuration: " .. config_file)
/// log.debug("Variable state: x=" .. tostring(x))
/// ```
fn log_debug(_: &Lua, msg: String) -> LuaResult<()> {
    log::debug(&msg);
    Ok(())
}

/// Logs a warning message.
/// 
/// This function logs a message at the WARN level, which is used for potentially
/// harmful situations or deprecated functionality. Warning messages indicate
/// that something unexpected happened or might happen, but the application
/// can still continue running.
/// 
/// # Arguments
/// 
/// * `msg` - The message to log
/// 
/// # Example
/// 
/// ```lua
/// log.warn("Using deprecated configuration option")
/// log.warn("Retrying failed operation")
/// ```
fn log_warn(_: &Lua, msg: String) -> LuaResult<()> {
    log::warn(&msg);
    Ok(())
}

/// Logs an error message.
/// 
/// This function logs a message at the ERROR level, which is used for error
/// events that might still allow the application to continue running. Error
/// messages indicate serious problems that need attention and should be
/// investigated and resolved.
/// 
/// # Arguments
/// 
/// * `msg` - The message to log
/// 
/// # Example
/// 
/// ```lua
/// log.error("Failed to connect to database")
/// log.error("Test suite failed with " .. error_count .. " errors")
/// ```
fn log_error(_: &Lua, msg: String) -> LuaResult<()> {
    log::error(&msg);
    Ok(())
}

/// Logs a trace message.
/// 
/// This function logs a message at the TRACE level, which is the most verbose
/// logging level. Trace messages provide very detailed information about the
/// execution flow and are typically used for fine-grained debugging. They
/// are usually filtered out in all but the most verbose logging configurations.
/// 
/// # Arguments
/// 
/// * `msg` - The message to log
/// 
/// # Example
/// 
/// ```lua
/// log.trace("Entering function: process_file()")
/// log.trace("Loop iteration " .. i .. " of " .. total)
/// ```
fn log_trace(_: &Lua, msg: String) -> LuaResult<()> {
    log::trace(&msg);
    Ok(())
}

/// Sets a custom log format template.
/// 
/// This function allows Lua scripts to configure the log output format using
/// a template string with placeholders for various log fields.
/// 
/// # Arguments
/// 
/// * `template` - The format template string with placeholders
/// 
/// # Supported Placeholders
/// 
/// - `{timestamp}` - Full timestamp in ISO format
/// - `{timestamp:format}` - Custom timestamp format (e.g., `{timestamp:%Y-%m-%d %H:%M:%S}`)
/// - `{level}` - Log level in uppercase
/// - `{level:lower}` - Log level in lowercase
/// - `{message}` - The log message content
/// - `{user}` - User identifier (if available)
/// - `{pid}` - Process ID
/// - `{thread}` - Thread ID
/// - `{source}` - Source location (if available)
/// - `{field:name}` - Custom field value
/// 
/// # Example
/// 
/// ```lua
/// log.set_format("[{timestamp:%H:%M:%S}] [{level}] {message}")
/// log.info("Custom formatted message")  -- [14:30:15] [INFO] Custom formatted message
/// ```
fn set_format(_: &Lua, template: String) -> LuaResult<()> {
    let sink = ConfigurableStdoutSink::with_template(template);
    crate::log::set_logger(sink);
    Ok(())
}

/// Sets the log format to CI/CD friendly format.
/// 
/// This configures logging to use a format optimized for CI/CD pipelines
/// with timestamp, user, level, and message.
/// 
/// Format: `[timestamp] [user] [level] message`
/// 
/// # Example
/// 
/// ```lua
/// log.set_ci_format()
/// log.info("Pipeline started")  -- [2023-07-10 14:30:15] [unknown] [INFO] Pipeline started
/// ```
fn set_ci_format(_: &Lua, _: ()) -> LuaResult<()> {
    let sink = ConfigurableStdoutSink::ci_format();
    crate::log::set_logger(sink);
    Ok(())
}

/// Sets the log format to JSON output.
/// 
/// This configures logging to output structured JSON format, which is
/// ideal for log aggregation systems and programmatic processing.
/// 
/// # Example
/// 
/// ```lua
/// log.set_json_format()
/// log.info("JSON formatted message")
/// -- Output: {"timestamp":"2023-07-10T14:30:15Z","level":"info","message":"JSON formatted message",...}
/// ```
fn set_json_format(_: &Lua, _: ()) -> LuaResult<()> {
    let sink = ConfigurableStdoutSink::json_format();
    crate::log::set_logger(sink);
    Ok(())
}

/// Sets the log format to development/debugging format.
/// 
/// This configures logging with a compact format showing time, thread,
/// level, and message - optimized for development and debugging.
/// 
/// Format: `[HH:MM:SS] [thread] [LEVEL] message`
/// 
/// # Example
/// 
/// ```lua
/// log.set_dev_format()
/// log.debug("Development debug message")  -- [14:30:15] [ThreadId(1)] [DEBUG] Development debug message
/// ```
fn set_dev_format(_: &Lua, _: ()) -> LuaResult<()> {
    let sink = ConfigurableStdoutSink::dev_format();
    crate::log::set_logger(sink);
    Ok(())
}

/// Sets the log format to simple format.
/// 
/// This resets logging to the basic simple format with just level and message.
/// This is the default format used when no custom formatting is specified.
/// 
/// Format: `[LEVEL] message`
/// 
/// # Example
/// 
/// ```lua
/// log.set_simple_format()
/// log.info("Simple formatted message")  -- [INFO] Simple formatted message
/// ```
fn set_simple_format(_: &Lua, _: ()) -> LuaResult<()> {
    let sink = ConfigurableStdoutSink::simple();
    crate::log::set_logger(sink);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_log_module_creation() {
        let lua = Lua::new();
        let log_mod = Log::create(&lua).unwrap();
        
        // Test that all log functions are present
        assert!(log_mod.get::<mlua::Function>("info").is_ok());
        assert!(log_mod.get::<mlua::Function>("warn").is_ok());
        assert!(log_mod.get::<mlua::Function>("error").is_ok());
        assert!(log_mod.get::<mlua::Function>("debug").is_ok());
        assert!(log_mod.get::<mlua::Function>("trace").is_ok());
    }

    #[test]
    fn test_log_functions_callable() {
        let lua = Lua::new();
        let log_mod = Log::create(&lua).unwrap();
        
        // Test that functions can be called without error
        let info_fn: mlua::Function = log_mod.get("info").unwrap();
        let result = info_fn.call::<()>("test message".to_string());
        assert!(result.is_ok());
        
        let debug_fn: mlua::Function = log_mod.get("debug").unwrap();
        let result = debug_fn.call::<()>("debug message".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_formatting_functions() {
        let lua = Lua::new();
        let log_mod = Log::create(&lua).unwrap();
        
        // Test that all formatting functions are present
        assert!(log_mod.get::<mlua::Function>("set_format").is_ok());
        assert!(log_mod.get::<mlua::Function>("set_ci_format").is_ok());
        assert!(log_mod.get::<mlua::Function>("set_json_format").is_ok());
        assert!(log_mod.get::<mlua::Function>("set_dev_format").is_ok());
        assert!(log_mod.get::<mlua::Function>("set_simple_format").is_ok());
    }

    #[test]
    fn test_set_custom_format() {
        let lua = Lua::new();
        let log_mod = Log::create(&lua).unwrap();
        
        // Test that custom format can be set without error
        let set_format_fn: mlua::Function = log_mod.get("set_format").unwrap();
        let result = set_format_fn.call::<()>("[{level}] {message}".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_preset_formats() {
        let lua = Lua::new();
        let log_mod = Log::create(&lua).unwrap();
        
        // Test that all preset formats can be set without error
        let ci_format_fn: mlua::Function = log_mod.get("set_ci_format").unwrap();
        assert!(ci_format_fn.call::<()>(()).is_ok());
        
        let json_format_fn: mlua::Function = log_mod.get("set_json_format").unwrap();
        assert!(json_format_fn.call::<()>(()).is_ok());
        
        let dev_format_fn: mlua::Function = log_mod.get("set_dev_format").unwrap();
        assert!(dev_format_fn.call::<()>(()).is_ok());
        
        let simple_format_fn: mlua::Function = log_mod.get("set_simple_format").unwrap();
        assert!(simple_format_fn.call::<()>(()).is_ok());
    }
}