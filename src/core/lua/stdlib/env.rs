// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for environment variable management.
use mlua::{Lua, Table, Result as LuaResult};
use super::GaiaModule;

pub struct Env;

// GaiaModule implementation for the Env module
impl GaiaModule for Env {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let env_table = lua.create_table()?;
        env_table.set("get", lua.create_function(env_get)?)?;
        env_table.set("list",lua.create_function(list_env)?)?;
        Ok(env_table)
    }
}

// Get an environment variable by key
fn env_get(_: &Lua, key: String) -> LuaResult<Option<String>> {
    Ok(std::env::var(&key).ok())
}

// List all environment variables
fn list_env(lua: &Lua, _: ()) -> LuaResult<Table> {
    let vars = std::env::vars();
    let tbl = lua.create_table()?;
    for (k, v) in vars {
        tbl.set(k, v)?;
    }
    Ok(tbl)
}