
use mlua::{Lua, Table, Result as LuaResult};

pub mod log;
pub mod shell;
pub mod fs;
pub mod env;
pub mod assert;

pub trait GaiaModule {
    fn create(lua: &Lua) -> LuaResult<Table>;
}

pub fn register_gaia_lib(lua: &Lua) -> LuaResult<()> {
    let gaia = lua.create_table()?;

    gaia.set("log", log::Log::create(lua)?)?;
    gaia.set("shell", shell::Shell::create(lua)?)?;
    gaia.set("env", env::Env::create(lua)?)?;
    gaia.set("fs", fs::Fs::create(lua)?)?;
    gaia.set("assert", assert::Assert::create(lua)?)?;

    lua.globals().set("gaia", gaia)?;
    Ok(())
}