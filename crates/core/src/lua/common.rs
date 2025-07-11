//! # Common Types and Traits for Lua Integration
//! 
//! This module contains shared types, traits, and utilities used across
//! all GaiaCI Lua standard library modules. It provides the foundation
//! for consistent module design and registration patterns.

use mlua::{Lua, Table, Result as LuaResult};

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
/// # use gaiaci_core::lua::common::GaiaModule;
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

/// Common result type for Lua operations that may fail.
/// 
/// This type alias provides a convenient shorthand for the most common
/// Result type used in Lua module implementations.
pub type ModuleResult<T> = LuaResult<T>;

/// Standard module creation helper.
/// 
/// This function provides a standardized way to create Lua tables
/// for modules, ensuring consistent initialization patterns.
/// 
/// # Arguments
/// 
/// * `lua` - The Lua context
/// * `capacity` - Optional hint for the expected number of functions
/// 
/// # Returns
/// 
/// A new Lua table ready for function registration.
pub fn create_module_table(lua: &Lua, capacity: Option<usize>) -> LuaResult<Table> {
    match capacity {
        Some(cap) => lua.create_table_with_capacity(0, cap),
        None => lua.create_table(),
    }
}
