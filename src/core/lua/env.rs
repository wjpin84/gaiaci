//! # Environment Module
//! 
//! This module provides environment variable access functions for Lua scripts in GaiaCI.
//! It allows CI pipelines to read environment variables and access system configuration,
//! which is essential for handling secrets, configuration parameters, and runtime settings.
//! 
//! ## Available Functions
//! 
//! - `get(key)` - Get the value of a specific environment variable
//! - `list()` - Get all environment variables as a table
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Get a specific environment variable
//! local home = env.get("HOME")
//! local path = env.get("PATH")
//! 
//! -- Get all environment variables
//! local all_vars = env.list()
//! for key, value in pairs(all_vars) do
//!     print(key .. " = " .. value)
//! end
//! 
//! -- Check if a variable exists
//! local api_key = env.get("API_KEY")
//! if api_key then
//!     -- Use the API key
//! else
//!     -- Handle missing API key
//! end
//! ```

use mlua::{Lua, Table, Result as LuaResult};
use super::GaiaModule;

/// The Environment module provides access to system environment variables.
/// 
/// This struct implements the GaiaModule trait to expose environment variable
/// functions to Lua environments, enabling scripts to access system configuration
/// and runtime parameters.
pub struct Env;

impl GaiaModule for Env {
    /// Creates a new Environment module table with all environment functions.
    /// 
    /// This method registers all available environment variable functions with the
    /// Lua environment, making them accessible under the `env` namespace.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context to create the module in
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing all environment functions
    fn create(lua: &Lua) -> LuaResult<Table> {
        let env_table = lua.create_table()?;
        env_table.set("get", lua.create_function(env_get)?)?;
        env_table.set("list", lua.create_function(list_env)?)?;
        Ok(env_table)
    }
}

/// Gets the value of a specific environment variable.
/// 
/// This function retrieves the value of an environment variable by its key name.
/// If the variable exists, its value is returned as a string. If the variable
/// does not exist, nil is returned.
/// 
/// # Arguments
/// 
/// * `key` - The name of the environment variable to retrieve
/// 
/// # Returns
/// 
/// * `Some(String)` if the environment variable exists
/// * `None` if the environment variable does not exist
/// 
/// # Example
/// 
/// ```lua
/// local home = env.get("HOME")        -- Returns "/home/user" or nil
/// local path = env.get("PATH")        -- Returns the PATH variable or nil
/// local missing = env.get("NOT_SET")  -- Returns nil
/// ```
fn env_get(_: &Lua, key: String) -> LuaResult<Option<String>> {
    Ok(std::env::var(&key).ok())
}

/// Lists all environment variables as a Lua table.
/// 
/// This function retrieves all environment variables available to the current
/// process and returns them as a Lua table where keys are variable names and
/// values are their corresponding values.
/// 
/// # Returns
/// 
/// A Lua table containing all environment variables as key-value pairs
/// 
/// # Example
/// 
/// ```lua
/// local all_env = env.list()
/// for key, value in pairs(all_env) do
///     print(key .. " = " .. value)
/// end
/// 
/// -- Access specific variables from the table
/// local home = all_env["HOME"]
/// local path = all_env["PATH"]
/// ```
fn list_env(lua: &Lua, _: ()) -> LuaResult<Table> {
    let vars = std::env::vars();
    let tbl = lua.create_table()?;
    for (k, v) in vars {
        tbl.set(k, v)?;
    }
    Ok(tbl)
}