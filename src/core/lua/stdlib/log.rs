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
    use std::sync::{Arc, Mutex};
    use crate::core::log::{set_logger, LogSink, LogLevel};

    struct TestLogger {
        entries: Arc<Mutex<Vec<(LogLevel, String)>>>,
    }

    impl LogSink for TestLogger {
        fn log(&self, level: LogLevel, msg: &str) {
            self.entries.lock().unwrap().push((level, msg.to_string()));
        }
    }

    macro_rules! test_log_level {
        ($name:ident, $level:ident, $lua:expr) => {
            #[test]
            fn $name() -> mlua::Result<()> {
                let lua = Lua::new();
                let entries = Arc::new(Mutex::new(Vec::new()));
                let logger = TestLogger { entries: Arc::clone(&entries) };
                set_logger(logger);

                let log_mod = Log::create(&lua)?;
                lua.globals().set("log", log_mod)?;
                lua.load($lua).exec()?;

                let captured = entries.lock().unwrap();
                assert_eq!(captured.len(), 1);
                assert_eq!(captured[0].0, LogLevel::$level);
                assert_eq!(captured[0].1, "hello");

                Ok(())
            }
        };
    }

    test_log_level!(test_log_info, Info, r#"log.info('hello')"#);
    test_log_level!(test_log_warn, Warn, r#"log.warn('hello')"#);
    test_log_level!(test_log_error, Error, r#"log.error('hello')"#);
    test_log_level!(test_log_debug, Debug, r#"log.debug('hello')"#);
    test_log_level!(test_log_trace, Trace, r#"log.trace('hello')"#);
}