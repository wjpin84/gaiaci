// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for environment variable management.
use mlua::{Lua, Result as LuaResult, Table};
use super::GaiaModule;
use crate::core::log;

pub struct Log;

// GaiaModule implementation for the Log module
impl GaiaModule for Log {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let log_table  = lua.create_table()?;
        log_table.set("info", lua.create_function(log_info)?)?;
        log_table.set("warn", lua.create_function(log_warn)?)?;
        log_table.set("error", lua.create_function(log_error)?)?;
        log_table.set("debug", lua.create_function(log_debug)?)?;
        log_table.set("trace", lua.create_function(log_trace)?)?;
        Ok(log_table)
    }
}

// Log a info message
fn log_info(_: &Lua, msg: String) -> LuaResult<()> {
    log::info(&msg);
    Ok(())
}

// Log a debug message
fn log_debug(_: &Lua, msg: String) -> LuaResult<()> {
    log::debug(&msg);
    Ok(())
}

// Log a warning message
fn log_warn(_: &Lua, msg: String) -> LuaResult<()> {
    log::warn(&msg);
    Ok(())
}

// Log an error message
fn log_error(_: &Lua, msg: String) -> LuaResult<()> {
    log::error(&msg);
    Ok(())
}

// Log a trace message
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