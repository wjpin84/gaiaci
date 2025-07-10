use mlua::{Lua, Result as LuaResult, Table};
use tracing::{info, warn, error, debug};
use super::GaiaModule;

pub struct Log;

impl GaiaModule for Log {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let log_table  = lua.create_table()?;
        log_table.set("info", lua.create_function(log_info)?)?;
        log_table.set("warn", lua.create_function(log_warn)?)?;
        log_table.set("error", lua.create_function(log_error)?)?;
        log_table.set("debug", lua.create_function(log_debug)?)?;
        Ok(log_table)
    }
}

fn log_info(_: &Lua, msg: String) -> LuaResult<()> {
    info!("{}", msg);
    Ok(())
}

fn log_debug(_: &Lua, msg: String) -> LuaResult<()> {
    debug!("{}", msg);
    Ok(())
}

fn log_warn(_: &Lua, msg: String) -> LuaResult<()> {
    warn!("{}", msg);
    Ok(())
}

fn log_error(_: &Lua, msg: String) -> LuaResult<()> {
    error!("{}", msg);
    Ok(())
}
