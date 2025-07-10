use gaiaci::core::lua::assert::Assert;
use gaiaci::core::lua::shell::Shell;
use gaiaci::core::lua::log::Log;
use gaiaci::core::lua::fs::Fs;
use gaiaci::core::lua::GaiaModule;
use mlua::{Lua, Result as LuaResult, Value};

/// Creates a Lua context with all GaiaCI modules loaded for testing
pub fn create_test_lua() -> LuaResult<Lua> {
    let lua = Lua::new();
    
    // Load standard libraries
    lua.load_std_libs(mlua::StdLib::ALL_SAFE)?;
    
    // Load GaiaCI modules
    let assert_module = Assert::create(&lua)?;
    lua.globals().set("assert", assert_module)?;
    
    let shell_module = Shell::create(&lua)?;
    lua.globals().set("shell", shell_module)?;
    
    let log_module = Log::create(&lua)?;
    lua.globals().set("log", log_module)?;
    
    let fs_module = Fs::create(&lua)?;
    lua.globals().set("fs", fs_module)?;
    
    Ok(lua)
}

/// Executes Lua code and returns the result
pub fn execute_lua_code(lua: &Lua, code: &str) -> LuaResult<Value> {
    lua.load(code).eval()
}

/// Helper to execute Lua code that should succeed
pub fn execute_lua_success(lua: &Lua, code: &str) -> Value {
    execute_lua_code(lua, code).expect("Lua code should execute successfully")
}

/// Helper to execute Lua code that should fail
pub fn execute_lua_error(lua: &Lua, code: &str) -> String {
    match execute_lua_code(lua, code) {
        Ok(_) => panic!("Expected Lua code to fail"),
        Err(e) => e.to_string(),
    }
}
