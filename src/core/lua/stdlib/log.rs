//! # Log Module
//! 
//! This module provides logging functions for Lua scripts in GaiaCI pipelines.
//! It integrates with GaiaCI's centralized logging system to provide structured
//! logging at various levels, enabling proper monitoring and debugging of CI processes.
//! 
//! ## Available Functions
//! 
//! - `trace(message)` - Log a trace-level message (most verbose)
//! - `debug(message)` - Log a debug-level message (verbose, for development)
//! - `info(message)` - Log an informational message (normal operation)
//! - `warn(message)` - Log a warning message (potential issues)
//! - `error(message)` - Log an error message (problems requiring attention)
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
//! -- Conditional logging
//! if some_condition then
//!     log.info("Condition met, proceeding with operation")
//! else
//!     log.warn("Condition not met, using fallback")
//! end
//! ```

use mlua::{Lua, Result as LuaResult, Table};
use super::GaiaModule;
use crate::core::log;

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
        log_table.set("info", lua.create_function(log_info)?)?;
        log_table.set("warn", lua.create_function(log_warn)?)?;
        log_table.set("error", lua.create_function(log_error)?)?;
        log_table.set("debug", lua.create_function(log_debug)?)?;
        log_table.set("trace", lua.create_function(log_trace)?)?;
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
}