//! # Lua Integration Module
//! 
//! This module provides comprehensive Lua scripting capabilities for GaiaCI,
//! enabling powerful and flexible automation of continuous integration pipelines.
//! The module integrates the mlua library with custom standard library extensions
//! specifically designed for CI/CD workflows.
//! 
//! ## Architecture
//! 
//! The Lua integration is built around a standard library that provides
//! CI-focused modules accessible from Lua scripts through the global `gaia` namespace.
//! 
//! ### Standard Library Modules
//! 
//! The standard library contains specialized modules for common CI operations:
//! 
//! - **`gaia.log`** - Structured logging with multiple severity levels
//! - **`gaia.shell`** - Execute system commands with environment control
//! - **`gaia.fs`** - File system operations (read, write, copy, list, etc.)
//! - **`gaia.env`** - Environment variable access and management
//! - **`gaia.assert`** - Comprehensive assertion functions for testing
//! 
//! ## Integration Pattern
//! 
//! All standard library modules implement the `GaiaModule` trait, providing
//! a consistent interface for registration and initialization within the
//! Lua environment.
//! 
//! ## Usage Example
//! 
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use mlua::Lua;
//! use gaiaci_core::lua::register_gaia_lib;
//! 
//! // Create Lua environment
//! let lua = Lua::new();
//! 
//! // Register GaiaCI standard library
//! register_gaia_lib(&lua)?;
//! 
//! // Execute Lua script with GaiaCI capabilities
//! lua.load(r#"
//!     -- Log pipeline start
//!     gaia.log.info("Starting CI pipeline")
//!     
//!     -- Check if build directory exists
//!     if gaia.fs.exists("build") then
//!         gaia.log.info("Build directory found")
//!         
//!         -- List build artifacts
//!         local files = gaia.fs.list_dir("build")
//!         for i, file in ipairs(files) do
//!             gaia.log.info("Artifact: " .. file)
//!         end
//!     else
//!         gaia.log.warn("Build directory not found, creating...")
//!         gaia.shell.run({cmd = "mkdir -p build"})
//!     end
//!     
//!     -- Run tests
//!     local result = gaia.shell.run({
//!         cmd = "npm test",
//!         capture = true,
//!         cwd = ".",
//!         env = {NODE_ENV = "test"}
//!     })
//!     
//!     if result.success then
//!         gaia.log.info("Tests passed!")
//!     else
//!         gaia.log.error("Tests failed: " .. result.stderr)
//!     end
//! "#).exec()?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Design Benefits
//! 
//! - **Declarative Pipelines** - Write CI logic in clear, readable Lua scripts
//! - **Dynamic Behavior** - Full programming language capabilities for complex logic
//! - **Error Handling** - Lua's error handling integrates with Rust's Result types
//! - **Sandboxing** - Controlled execution environment for security
//! - **Extensibility** - Easy to add new modules and functionality
//! 
//! ## Security Considerations
//! 
//! The Lua environment provides powerful system access through shell commands
//! and file operations. In production deployments, consider:
//! 
//! - Input validation for user-provided scripts
//! - Sandboxing mechanisms to limit system access
//! - Resource limits to prevent infinite loops or excessive memory usage
//! - Audit logging of executed operations

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
/// # use gaiaci_core::lua::GaiaModule;
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
/// use gaiaci_core::lua::register_gaia_lib;
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
