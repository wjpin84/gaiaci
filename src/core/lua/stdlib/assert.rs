//! # Assert Module
//! 
//! This module provides comprehensive assertion functions for Lua scripts in GaiaCI.
//! It implements a variety of assertion types commonly needed in continuous integration
//! pipelines, including value equality, null checks, string operations, and table validation.
//! 
//! ## Available Functions
//! 
//! - `equals(a, b)` - Assert that two values are equal
//! - `not_nil(value)` - Assert that a value is not nil
//! - `is_nil(value)` - Assert that a value is nil
//! - `is_true(condition)` - Assert that a boolean condition is true
//! - `str_contains(haystack, needle)` - Assert that a string contains a substring
//! - `str_matches(text, pattern)` - Assert that a string matches a regex pattern
//! - `table_has_key(table, key)` - Assert that a table contains a specific key
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Basic equality and nil checks
//! assert.equals(42, 42)
//! assert.not_nil("hello")
//! assert.is_nil(nil)
//! assert.is_true(true)
//! 
//! -- String assertions
//! assert.str_contains("hello world", "world")
//! assert.str_matches("test123", "test\\d+")
//! 
//! -- Table assertions
//! local table = {key1 = "value1"}
//! assert.table_has_key(table, "key1")
//! ```

use mlua::{Lua, Result as LuaResult, Table, Value, Error};
use super::GaiaModule;
use regex;

/// The Assert module provides assertion functions for Lua scripts.
/// 
/// This struct implements the GaiaModule trait to expose assertion functions
/// to Lua environments, enabling robust testing and validation within CI pipelines.
pub struct Assert;

impl GaiaModule for Assert {
    /// Creates a new Assert module table with all assertion functions.
    /// 
    /// This method registers all available assertion functions with the Lua environment,
    /// making them accessible under the `assert` namespace.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context to create the module in
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing all assertion functions
    fn create(lua: &Lua) -> LuaResult<Table> {
        let tbl = lua.create_table()?;
        tbl.set("equals", lua.create_function(assert_equals)?)?;
        tbl.set("not_nil", lua.create_function(assert_not_nil)?)?;
        tbl.set("is_nil", lua.create_function(assert_nil)?)?;
        tbl.set("is_true", lua.create_function(assert_true)?)?;
        tbl.set("str_contains", lua.create_function(assert_str_contains)?)?;
        tbl.set("str_matches", lua.create_function(assert_str_matches)?)?;
        tbl.set("table_has_key", lua.create_function(assert_table_has_key)?)?;
        Ok(tbl)
    }
}

/// Asserts that two values are equal.
/// 
/// This function compares two Lua values for equality using Lua's built-in
/// comparison semantics. It supports all Lua value types including numbers,
/// strings, booleans, tables, and nil.
/// 
/// # Arguments
/// 
/// * `a` - The first value to compare
/// * `b` - The second value to compare
/// 
/// # Returns
/// 
/// * `Ok(())` if the values are equal
/// * `Err(RuntimeError)` if the values are not equal
/// 
/// # Example
/// 
/// ```lua
/// assert.equals(42, 42)        -- passes
/// assert.equals("a", "a")      -- passes
/// assert.equals(1, "1")        -- fails - different types
/// ```
fn assert_equals(_: &Lua, (a, b): (Value, Value)) -> LuaResult<()> {
    if a != b {
        Err(Error::RuntimeError(format!("Assertion failed: {:?} != {:?}", a, b)))
    } else {
        Ok(())
    }
}

/// Asserts that a value is not nil.
/// 
/// This function verifies that the provided value is not nil, which is useful
/// for checking that required values are present or that function calls
/// returned valid results.
/// 
/// # Arguments
/// 
/// * `val` - The value to check for non-nil status
/// 
/// # Returns
/// 
/// * `Ok(())` if the value is not nil
/// * `Err(RuntimeError)` if the value is nil
/// 
/// # Example
/// 
/// ```lua
/// assert.not_nil("hello")      -- passes
/// assert.not_nil(42)           -- passes
/// assert.not_nil(nil)          -- fails
/// ```
fn assert_not_nil(_: &Lua, val: Value) -> LuaResult<()> {
    if matches!(val, Value::Nil) {
        Err(Error::RuntimeError("Assertion failed: value is nil".to_string()))
    } else {
        Ok(())
    }
}

/// Asserts that a value is nil.
/// 
/// This function verifies that the provided value is nil, which is useful
/// for checking that optional values are absent or that cleanup operations
/// have properly cleared values.
/// 
/// # Arguments
/// 
/// * `val` - The value to check for nil status
/// 
/// # Returns
/// 
/// * `Ok(())` if the value is nil
/// * `Err(RuntimeError)` if the value is not nil
/// 
/// # Example
/// 
/// ```lua
/// assert.is_nil(nil)           -- passes
/// assert.is_nil("hello")       -- fails
/// assert.is_nil(42)            -- fails
/// ```
fn assert_nil(_: &Lua, val: Value) -> LuaResult<()> {
    if !matches!(val, Value::Nil) {
        Err(Error::RuntimeError("Assertion failed: value is not nil".to_string()))
    } else {
        Ok(())
    }
}

/// Asserts that a boolean condition is true.
/// 
/// This function verifies that the provided boolean value is true, which is
/// fundamental for testing conditional logic and validation in CI pipelines.
/// 
/// # Arguments
/// 
/// * `cond` - The boolean condition to evaluate
/// 
/// # Returns
/// 
/// * `Ok(())` if the condition is true
/// * `Err(RuntimeError)` if the condition is false
/// 
/// # Example
/// 
/// ```lua
/// assert.is_true(true)         -- passes
/// assert.is_true(1 > 0)        -- passes
/// assert.is_true(false)        -- fails
/// ```
fn assert_true(_: &Lua, cond: bool) -> LuaResult<()> {
    if !cond {
        Err(Error::RuntimeError("Assertion failed: condition is false".to_string()))
    } else {
        Ok(())
    }
}

/// Asserts that one string contains another string.
/// 
/// This function performs a substring search to verify that the haystack
/// string contains the needle string. This is useful for validating log
/// output, command results, or file contents in CI scenarios.
/// 
/// # Arguments
/// 
/// * `haystack` - The string to search within (must be a string value)
/// * `needle` - The substring to search for (must be a string value)
/// 
/// # Returns
/// 
/// * `Ok(())` if the haystack contains the needle
/// * `Err(RuntimeError)` if the substring is not found or arguments are not strings
/// 
/// # Example
/// 
/// ```lua
/// assert.str_contains("hello world", "world")     -- passes
/// assert.str_contains("rustacean", "rust")        -- passes
/// assert.str_contains("foo", "bar")                -- fails
/// ```
fn assert_str_contains(_: &Lua, (haystack, needle): (Value, Value)) -> LuaResult<()> {
    match (&haystack, &needle) {
        (Value::String(h), Value::String(n)) => {
            let h_str = h.to_str()?;
            let n_str = n.to_str()?;
            if h_str.contains(&*n_str) {
                Ok(())
            } else {
                Err(Error::RuntimeError(format!("Assertion failed: '{}' does not contain '{}'", h_str, n_str)))
            }
        }
        _ => Err(Error::RuntimeError("Assertion failed: both arguments must be strings".to_string())),
    }
}

/// Asserts that a string matches a regular expression pattern.
/// 
/// This function uses Rust's regex engine to verify that the provided text
/// matches the given regular expression pattern. This is particularly useful
/// for validating formatted output, version strings, or structured data.
/// 
/// # Arguments
/// 
/// * `text` - The string to test against the pattern (must be a string value)
/// * `pattern` - The regular expression pattern (must be a string value)
/// 
/// # Returns
/// 
/// * `Ok(())` if the text matches the pattern
/// * `Err(RuntimeError)` if the text doesn't match, pattern is invalid, or arguments are not strings
/// 
/// # Example
/// 
/// ```lua
/// assert.str_matches("test123", "test\\d+")        -- passes
/// assert.str_matches("v1.2.3", "v\\d+\\.\\d+\\.\\d+")  -- passes
/// assert.str_matches("hello", "\\d+")              -- fails
/// ```
fn assert_str_matches(_: &Lua, (text, pattern): (Value, Value)) -> LuaResult<()> {
    match (&text, &pattern) {
        (Value::String(t), Value::String(p)) => {
            let t_str = t.to_str()?;
            let p_str = p.to_str()?;
            let re = regex::Regex::new(&*p_str)
                .map_err(|e| Error::RuntimeError(format!("Invalid regex: {}", e)))?;
            if re.is_match(&*t_str) {
                Ok(())
            } else {
                Err(Error::RuntimeError(format!("Assertion failed: '{}' does not match regex '{}'", t_str, p_str)))
            }
        }
        _ => Err(Error::RuntimeError("Assertion failed: both arguments must be strings".to_string())),
    }
}

/// Asserts that a table contains a specific key.
/// 
/// This function verifies that the provided table contains the specified key,
/// which is useful for validating configuration objects, API responses, or
/// data structures in CI pipelines.
/// 
/// # Arguments
/// 
/// * `table` - The table to search (must be a table value)
/// * `key` - The key to search for (can be any Lua value)
/// 
/// # Returns
/// 
/// * `Ok(())` if the table contains the key
/// * `Err(RuntimeError)` if the key is not found or the first argument is not a table
/// 
/// # Example
/// 
/// ```lua
/// local config = {name = "test", version = "1.0"}
/// assert.table_has_key(config, "name")       -- passes
/// assert.table_has_key(config, "version")    -- passes
/// assert.table_has_key(config, "missing")    -- fails
/// ```
fn assert_table_has_key(_lua: &Lua, (table, key): (Value, Value)) -> LuaResult<()> {
    if let Value::Table(t) = table {
        if t.contains_key(key)? {
            Ok(())
        } else {
            Err(Error::RuntimeError("Assertion failed: table missing key".to_string()))
        }
    } else {
        Err(Error::RuntimeError("Assertion failed: not a table".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, Value};

    #[test]
    fn test_assert_equals_basic() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("equals").unwrap();
        
        // Test success case
        f.call::<()>((Value::Integer(42), Value::Integer(42))).unwrap();
        
        // Test failure case
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(42), Value::Integer(43)));
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_not_nil_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("not_nil").unwrap();
        f.call::<()>(Value::String(lua.create_string("hello").unwrap())).unwrap();
    }

    #[test]
    fn test_assert_not_nil_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("not_nil").unwrap();
        let result: mlua::Result<()> = f.call::<()>(Value::Nil);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("value is nil"));
    }

    #[test]
    fn test_assert_nil_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("is_nil").unwrap();
        let result: mlua::Result<()> = f.call::<()>(Value::String(lua.create_string("hello").unwrap()));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("value is not nil"));
    }

    #[test]
    fn test_assert_nil_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("is_nil").unwrap();
        f.call::<()>(Value::Nil).unwrap();
    }

    #[test]
    fn test_assert_is_true_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("is_true").unwrap();
        f.call::<()>(true).unwrap();
    }

    #[test]
    fn test_assert_is_true_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("is_true").unwrap();
        let result: mlua::Result<()> = f.call::<()>(false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("condition is false"));
    }

    #[test]
    fn test_assert_equals_type_mismatch() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("equals").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(1), Value::String(lua.create_string("1").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_str_contains_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_contains").unwrap();
        f.call::<()>((Value::String(lua.create_string("rustacean").unwrap()), Value::String(lua.create_string("rust").unwrap()))).unwrap();
    }

    #[test]
    fn test_assert_str_contains_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_contains").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::String(lua.create_string("rustacean").unwrap()), Value::String(lua.create_string("c++").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_str_contains_type_mismatch() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_contains").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(1), Value::String(lua.create_string("1").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_str_matches_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_matches").unwrap();
        f.call::<()>((Value::String(lua.create_string("hello123").unwrap()), Value::String(lua.create_string(r"hello\d+").unwrap()))).unwrap();
    }

    #[test]
    fn test_assert_str_matches_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_matches").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::String(lua.create_string("hello123").unwrap()), Value::String(lua.create_string(r"world\d+").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_str_matches_invalid_regex() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_matches").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::String(lua.create_string("hello123").unwrap()), Value::String(lua.create_string(r"hello[").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid regex"));
    }

    #[test]
    fn test_assert_str_matches_type_mismatch() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_matches").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(1), Value::String(lua.create_string("1").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_table_has_key_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("table_has_key").unwrap();
        let table = lua.create_table().unwrap();
        table.set("key1", 42).unwrap();
        f.call::<()>((Value::Table(table.clone()), Value::String(lua.create_string("key1").unwrap()))).unwrap();
    }

    #[test]
    fn test_assert_table_has_key_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("table_has_key").unwrap();
        let table = lua.create_table().unwrap();
        table.set("key1", 42).unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Table(table.clone()), Value::String(lua.create_string("key2").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("table missing key"));
    }

    #[test]
    fn test_assert_table_has_key_type_mismatch() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("table_has_key").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(1), Value::String(lua.create_string("1").unwrap())));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
    }
}
