//! # GaiaCI Lua Standard Library
//! 
//! This module implements a comprehensive standard library for Lua scripts
//! in GaiaCI pipelines. It provides specialized modules that expose essential
//! CI/CD functionality through a clean, consistent Lua API.
//! 
//! ## Module Architecture
//! 
//! All standard library modules implement the `GaiaModule` trait, which provides
//! a uniform interface for creating and registering Lua table modules. This
//! ensures consistent behavior and integration patterns across all modules.
//! 
//! ### Available Modules
//! 
//! #### **Logging** (`gaia.log`)
//! Structured logging with multiple severity levels:
//! ```lua
//! gaia.log.info("Pipeline started")
//! gaia.log.warn("Configuration file missing")
//! gaia.log.error("Build failed")
//! gaia.log.debug("Variable x = " .. tostring(x))
//! gaia.log.trace("Entering function calculate_metrics")
//! ```
//! 
//! #### **Shell Commands** (`gaia.shell`)
//! Execute system commands with full environment control:
//! ```lua
//! local result = gaia.shell.run({
//!     cmd = "npm install",
//!     capture = true,
//!     cwd = "/project",
//!     env = {NODE_ENV = "production"}
//! })
//! ```
//! 
//! #### **File System** (`gaia.fs`)
//! Comprehensive file and directory operations:
//! ```lua
//! local config = gaia.fs.read("config.json")
//! gaia.fs.write("output.txt", "Build completed")
//! local files = gaia.fs.list_dir("build")
//! gaia.fs.copy("app.jar", "releases/v1.0.0.jar")
//! ```
//! 
//! #### **Environment Variables** (`gaia.env`)
//! Access system environment variables:
//! ```lua
//! local home = gaia.env.get("HOME")
//! local all_vars = gaia.env.list()
//! ```
//! 
//! #### **Assertions** (`gaia.assert`)
//! Testing and validation functions:
//! ```lua
//! gaia.assert.equals(actual, expected)
//! gaia.assert.str_contains(output, "SUCCESS")
//! gaia.assert.table_has_key(config, "version")
//! ```
//! 
//! ## Registration and Usage
//! 
//! The `register_gaia_lib` function sets up the complete standard library
//! in a Lua environment, making all modules available under the global
//! `gaia` namespace.
//! 
//! ## Design Principles
//! 
//! - **Consistency** - All modules follow the same patterns and conventions
//! - **CI/CD Focus** - Functions designed specifically for automation workflows
//! - **Error Integration** - Rust errors propagate cleanly to Lua exceptions
//! - **Type Safety** - Strong typing with clear error messages
//! - **Performance** - Efficient implementations suitable for CI workloads

use mlua::{Lua, Table, Result as LuaResult};

pub mod log;
pub mod shell;
pub mod fs;
pub mod env;
pub mod assert;

/// Trait for GaiaCI Lua standard library modules.
/// 
/// This trait provides a consistent interface for creating Lua modules
/// that integrate with the GaiaCI standard library. All standard library
/// modules implement this trait to ensure uniform registration and
/// initialization patterns.
/// 
/// ## Implementation Requirements
/// 
/// Implementors must provide a `create` function that:
/// - Takes a Lua context reference
/// - Returns a configured Lua table containing module functions
/// - Handles any initialization errors appropriately
/// 
/// ## Example Implementation
/// 
/// ```rust
/// # use mlua::{Lua, Table, Result as LuaResult};
/// # use gaiaci::core::lua::stdlib::GaiaModule;
/// # 
/// # fn my_function_impl(_lua: &Lua, _args: ()) -> LuaResult<()> { Ok(()) }
/// 
/// pub struct MyModule;
/// 
/// impl GaiaModule for MyModule {
///     fn create(lua: &Lua) -> LuaResult<Table> {
///         let table = lua.create_table()?;
///         table.set("my_function", lua.create_function(my_function_impl)?)?;
///         Ok(table)
///     }
/// }
/// ```
pub trait GaiaModule {
    /// Creates and configures a Lua table for this module.
    /// 
    /// This method is called during standard library registration to
    /// create the module's Lua table and populate it with functions.
    /// The returned table will be registered in the global `gaia`
    /// namespace under the module's name.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context in which to create the module
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing the configured module table
    /// with all module functions registered.
    /// 
    /// # Errors
    /// 
    /// Returns a Lua error if module creation or function registration fails.
    fn create(lua: &Lua) -> LuaResult<Table>;
}

/// Registers the complete GaiaCI standard library in a Lua environment.
/// 
/// This function creates the global `gaia` namespace and registers all
/// standard library modules within it. After calling this function,
/// Lua scripts can access all GaiaCI functionality through the `gaia`
/// global variable.
/// 
/// ## Registered Modules
/// 
/// - `gaia.log` - Logging functions
/// - `gaia.shell` - Shell command execution
/// - `gaia.fs` - File system operations
/// - `gaia.env` - Environment variable access
/// - `gaia.assert` - Assertion functions
/// 
/// # Arguments
/// 
/// * `lua` - The Lua context in which to register the standard library
/// 
/// # Returns
/// 
/// * `Ok(())` if registration succeeds
/// * `Err(LuaError)` if any module registration fails
/// 
/// # Example
/// 
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use mlua::Lua;
/// use gaiaci::core::lua::stdlib::register_gaia_lib;
/// 
/// let lua = Lua::new();
/// register_gaia_lib(&lua)?;
/// 
/// // Now Lua scripts can use: gaia.log.info(), gaia.fs.read(), etc.
/// lua.load("gaia.log.info('Standard library loaded!')").exec()?;
/// # Ok(())
/// # }
/// ```
/// 
/// # Error Handling
/// 
/// If any module fails to register, the entire registration process fails
/// and returns an error. This ensures that the standard library is either
/// completely available or not available at all, preventing partial
/// initialization states.
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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_register_gaia_lib() {
        let lua = Lua::new();
        
        // Register the GaiaCI standard library
        let result = register_gaia_lib(&lua);
        assert!(result.is_ok(), "Failed to register GaiaCI standard library");
    }

    #[test]
    fn test_gaia_namespace_exists() {
        let lua = Lua::new();
        register_gaia_lib(&lua).unwrap();
        
        // Check that the gaia global exists
        let result: mlua::Result<mlua::Table> = lua.globals().get("gaia");
        assert!(result.is_ok(), "gaia global namespace not found");
    }

    #[test]
    fn test_all_modules_registered() {
        let lua = Lua::new();
        register_gaia_lib(&lua).unwrap();
        
        let gaia: mlua::Table = lua.globals().get("gaia").unwrap();
        
        // Check that all expected modules are registered
        assert!(gaia.get::<mlua::Table>("log").is_ok(), "log module not registered");
        assert!(gaia.get::<mlua::Table>("shell").is_ok(), "shell module not registered");
        assert!(gaia.get::<mlua::Table>("fs").is_ok(), "fs module not registered");
        assert!(gaia.get::<mlua::Table>("env").is_ok(), "env module not registered");
        assert!(gaia.get::<mlua::Table>("assert").is_ok(), "assert module not registered");
    }

    #[test]
    fn test_module_functions_accessible() {
        let lua = Lua::new();
        register_gaia_lib(&lua).unwrap();
        
        // Test that we can access functions from each module
        let result = lua.load(r#"
            -- Test that all module functions are accessible
            local log_info = gaia.log.info
            local shell_run = gaia.shell.run
            local fs_exists = gaia.fs.exists
            local env_get = gaia.env.get
            local assert_equals = gaia.assert.equals
            
            -- Return true if all functions are accessible
            return type(log_info) == "function" and
                   type(shell_run) == "function" and
                   type(fs_exists) == "function" and
                   type(env_get) == "function" and
                   type(assert_equals) == "function"
        "#).eval::<bool>();
        
        assert!(result.is_ok(), "Failed to evaluate module function accessibility");
        assert!(result.unwrap(), "Not all module functions are accessible");
    }

    #[test]
    fn test_lua_script_execution() {
        let lua = Lua::new();
        register_gaia_lib(&lua).unwrap();
        
        // Test a simple Lua script that uses multiple modules
        let result = lua.load(r#"
            -- Test basic functionality of each module
            gaia.log.info("Testing GaiaCI standard library")
            
            local home_exists = gaia.fs.exists(".")
            if not home_exists then
                error("Current directory should exist")
            end
            
            local path_var = gaia.env.get("PATH")
            if not path_var then
                gaia.log.warn("PATH environment variable not found")
            end
            
            gaia.assert.equals(1, 1)
            gaia.assert.is_true(true)
            
            return "success"
        "#).eval::<String>();
        
        assert!(result.is_ok(), "Lua script execution failed: {:?}", result.err());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_multiple_registrations() {
        let lua = Lua::new();
        
        // Register twice to ensure it doesn't cause issues
        assert!(register_gaia_lib(&lua).is_ok());
        assert!(register_gaia_lib(&lua).is_ok());
        
        // Verify the library is still functional
        let gaia: mlua::Table = lua.globals().get("gaia").unwrap();
        assert!(gaia.get::<mlua::Table>("log").is_ok());
    }
}