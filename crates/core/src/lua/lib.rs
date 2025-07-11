//! # GaiaCI Standard Library Registration
//! 
//! This module contains the main registration logic for the GaiaCI Lua
//! standard library. It handles the creation of the global `gaia` namespace
//! and registration of all standard library modules.
//! 
//! ## Architecture
//! 
//! The standard library registration follows a modular approach where each
//! module implements the `GaiaModule` trait and can be registered independently.
//! This design allows for:
//! 
//! - Easy addition of new modules
//! - Selective module loading (if needed in the future)
//! - Consistent error handling across all modules
//! - Clear separation between module logic and registration logic
//! 
//! ## Usage
//! 
//! ```rust
//! # use mlua::Lua;
//! # use gaiaci_core::lua::lib::register_gaia_lib;
//! 
//! let lua = Lua::new();
//! register_gaia_lib(&lua)?;
//! 
//! // Now all gaia.* modules are available
//! lua.load("gaia.log.info('Hello from GaiaCI!')").exec()?;
//! # Ok::<(), mlua::Error>(())
//! ```

use mlua::{Lua, Result as LuaResult};
use crate::lua::common::GaiaModule;
use crate::lua::{log, shell, fs, env, assert};

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
/// use gaiaci_core::lua::lib::register_gaia_lib;
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
    let gaia = lua.create_table_with_capacity(0, 5)?; // Pre-allocate for 5 modules

    // Register all standard library modules
    gaia.set("log", log::Log::create(lua)?)?;
    gaia.set("shell", shell::Shell::create(lua)?)?;
    gaia.set("env", env::Env::create(lua)?)?;
    gaia.set("fs", fs::Fs::create(lua)?)?;
    gaia.set("assert", assert::Assert::create(lua)?)?;

    // Register the gaia namespace globally
    lua.globals().set("gaia", gaia)?;
    Ok(())
}

/// Registers a subset of modules (useful for testing or restricted environments).
/// 
/// This function allows selective registration of standard library modules,
/// which can be useful for testing individual modules or creating restricted
/// execution environments.
/// 
/// # Arguments
/// 
/// * `lua` - The Lua context
/// * `modules` - List of module names to register
/// 
/// # Returns
/// 
/// * `Ok(())` if all specified modules register successfully
/// * `Err(LuaError)` if any module registration fails
/// 
/// # Example
/// 
/// ```rust
/// # use mlua::Lua;
/// # use gaiaci_core::lua::lib::register_modules;
/// 
/// let lua = Lua::new();
/// // Only register logging and assertions for a test environment
/// register_modules(&lua, &["log", "assert"])?;
/// # Ok::<(), mlua::Error>(())
/// ```
pub fn register_modules(lua: &Lua, modules: &[&str]) -> LuaResult<()> {
    let gaia = lua.create_table_with_capacity(0, modules.len())?;

    for &module_name in modules {
        match module_name {
            "log" => gaia.set("log", log::Log::create(lua)?)?,
            "shell" => gaia.set("shell", shell::Shell::create(lua)?)?,
            "env" => gaia.set("env", env::Env::create(lua)?)?,
            "fs" => gaia.set("fs", fs::Fs::create(lua)?)?,
            "assert" => gaia.set("assert", assert::Assert::create(lua)?)?,
            _ => return Err(mlua::Error::RuntimeError(
                format!("Unknown module: {}", module_name)
            )),
        }
    }

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
    fn test_selective_module_registration() {
        let lua = Lua::new();
        register_modules(&lua, &["log", "assert"]).unwrap();
        
        let gaia: mlua::Table = lua.globals().get("gaia").unwrap();
        
        // Check that only specified modules are registered
        assert!(gaia.get::<mlua::Table>("log").is_ok(), "log module should be registered");
        assert!(gaia.get::<mlua::Table>("assert").is_ok(), "assert module should be registered");
        
        // These should not be registered
        assert!(gaia.get::<mlua::Table>("shell").is_err(), "shell module should not be registered");
        assert!(gaia.get::<mlua::Table>("fs").is_err(), "fs module should not be registered");
        assert!(gaia.get::<mlua::Table>("env").is_err(), "env module should not be registered");
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

    #[test]
    fn test_unknown_module_error() {
        let lua = Lua::new();
        
        // Try to register an unknown module
        let result = register_modules(&lua, &["log", "unknown_module"]);
        assert!(result.is_err(), "Should fail with unknown module");
        
        if let Err(mlua::Error::RuntimeError(msg)) = result {
            assert!(msg.contains("Unknown module: unknown_module"));
        } else {
            panic!("Expected RuntimeError with unknown module message");
        }
    }
}
