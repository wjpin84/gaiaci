//! # Assert Module
//! 
//! This module provides comprehensive assertion functions for Lua scripts in GaiaCI.
//! It implements a variety of assertion types commonly needed in continuous integration
//! pipelines, including value equality, null checks, string operations, and table validation.
//! 
//! ## Available Functions
//! 
//! All assertion functions accept an optional error message as the last parameter
//! to provide better debugging information when assertions fail.
//! 
//! - `equals(a, b, message?)` - Assert that two values are equal
//! - `not_nil(value, message?)` - Assert that a value is not nil
//! - `is_nil(value, message?)` - Assert that a value is nil
//! - `is_true(condition, message?)` - Assert that a boolean condition is true
//! - `str_contains(haystack, needle, message?)` - Assert that a string contains a substring
//! - `str_matches(text, pattern, message?)` - Assert that a string matches a regex pattern
//! - `table_has_key(table, key, message?)` - Assert that a table contains a specific key
//! - `command_exists(command, message?)` - Assert that a command/executable exists in PATH
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Basic assertions with default messages
//! assert.equals(42, 42)
//! assert.not_nil("hello")
//! 
//! -- Enhanced assertions with custom messages
//! assert.equals(user.age, 25, "User age should be 25")
//! assert.is_true(build_success, "Build should have succeeded")
//! assert.str_contains(log_output, "SUCCESS", "Log should contain success message")
//! 
//! -- Table assertions with context
//! assert.table_has_key(config, "database_url", "Config must specify database URL")
//! 
//! -- Command/tool availability assertions  
//! assert.command_exists("git", "Git is required for this pipeline")
//! assert.command_exists("docker", "Docker must be installed")
//! assert.command_exists("python")  -- Uses default error message
//! ```

use mlua::{Lua, Result as LuaResult, Table, Value, Error};
use super::GaiaModule;
use regex;

/// Helper function to create an assertion error with optional custom message.
/// 
/// # Arguments
/// 
/// * `default_msg` - The default error message to use
/// * `custom_msg` - Optional custom message from the user
/// 
/// # Returns
/// 
/// A formatted error message combining both messages if custom message is provided
fn create_assertion_error(default_msg: &str, custom_msg: Option<&str>) -> String {
    match custom_msg {
        Some(msg) => format!("{}: {}", msg, default_msg),
        None => default_msg.to_string(),
    }
}

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
        tbl.set("command_exists", lua.create_function(assert_command_exists)?)?;
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the values are equal
/// * `Err(RuntimeError)` if the values are not equal
/// 
/// # Example
/// 
/// ```lua
/// assert.equals(42, 42)                              -- passes
/// assert.equals("a", "a")                            -- passes
/// assert.equals(1, 2, "Numbers should be equal")     -- fails with custom message
/// assert.equals(1, "1")                              -- fails - different types
/// ```
fn assert_equals(_: &Lua, (a, b, message): (Value, Value, Option<Value>)) -> LuaResult<()> {
    if a != b {
        let custom_msg = message
            .and_then(|v| match v {
                Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                _ => None,
            });
        let default_msg = format!("Assertion failed: {:?} != {:?}", a, b);
        let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
        Err(Error::RuntimeError(error_msg))
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the value is not nil
/// * `Err(RuntimeError)` if the value is nil
/// 
/// # Example
/// 
/// ```lua
/// assert.not_nil("hello")                           -- passes
/// assert.not_nil(42)                               -- passes
/// assert.not_nil(nil, "Value should not be nil")   -- fails with custom message
/// ```
fn assert_not_nil(_: &Lua, (val, message): (Value, Option<Value>)) -> LuaResult<()> {
    if matches!(val, Value::Nil) {
        let custom_msg = message
            .and_then(|v| match v {
                Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                _ => None,
            });
        let default_msg = "Assertion failed: value is nil";
        let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
        Err(Error::RuntimeError(error_msg))
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the value is nil
/// * `Err(RuntimeError)` if the value is not nil
/// 
/// # Example
/// 
/// ```lua
/// assert.is_nil(nil)                                    -- passes
/// assert.is_nil("hello", "Expected value to be nil")   -- fails with custom message
/// assert.is_nil(42)                                     -- fails
/// ```
fn assert_nil(_: &Lua, (val, message): (Value, Option<Value>)) -> LuaResult<()> {
    if !matches!(val, Value::Nil) {
        let custom_msg = message
            .and_then(|v| match v {
                Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                _ => None,
            });
        let default_msg = "Assertion failed: value is not nil";
        let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
        Err(Error::RuntimeError(error_msg))
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the condition is true
/// * `Err(RuntimeError)` if the condition is false
/// 
/// # Example
/// 
/// ```lua
/// assert.is_true(true)                                   -- passes
/// assert.is_true(1 > 0)                                  -- passes
/// assert.is_true(false, "Condition should be true")     -- fails with custom message
/// ```
fn assert_true(_: &Lua, (cond, message): (bool, Option<Value>)) -> LuaResult<()> {
    if !cond {
        let custom_msg = message
            .and_then(|v| match v {
                Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                _ => None,
            });
        let default_msg = "Assertion failed: condition is false";
        let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
        Err(Error::RuntimeError(error_msg))
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the haystack contains the needle
/// * `Err(RuntimeError)` if the substring is not found or arguments are not strings
/// 
/// # Example
/// 
/// ```lua
/// assert.str_contains("hello world", "world")                    -- passes
/// assert.str_contains("rustacean", "rust")                      -- passes
/// assert.str_contains("foo", "bar", "Should contain 'bar'")     -- fails with custom message
/// ```
fn assert_str_contains(_: &Lua, (haystack, needle, message): (Value, Value, Option<Value>)) -> LuaResult<()> {
    let custom_msg = message
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        });

    match (&haystack, &needle) {
        (Value::String(h), Value::String(n)) => {
            let h_str = h.to_str()?;
            let n_str = n.to_str()?;
            if h_str.contains(&*n_str) {
                Ok(())
            } else {
                let default_msg = format!("Assertion failed: '{}' does not contain '{}'", h_str, n_str);
                let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
                Err(Error::RuntimeError(error_msg))
            }
        }
        _ => {
            let default_msg = "Assertion failed: both arguments must be strings";
            let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
            Err(Error::RuntimeError(error_msg))
        }
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
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the text matches the pattern
/// * `Err(RuntimeError)` if the text doesn't match, pattern is invalid, or arguments are not strings
/// 
/// # Example
/// 
/// ```lua
/// assert.str_matches("test123", "test\\d+")                             -- passes
/// assert.str_matches("v1.2.3", "v\\d+\\.\\d+\\.\\d+")                  -- passes
/// assert.str_matches("hello", "\\d+", "Should be numeric")              -- fails with custom message
/// ```
fn assert_str_matches(_: &Lua, (text, pattern, message): (Value, Value, Option<Value>)) -> LuaResult<()> {
    let custom_msg = message
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        });

    match (&text, &pattern) {
        (Value::String(t), Value::String(p)) => {
            let t_str = t.to_str()?;
            let p_str = p.to_str()?;
            let re = regex::Regex::new(&*p_str)
                .map_err(|e| {
                    let default_msg = format!("Invalid regex: {}", e);
                    let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
                    Error::RuntimeError(error_msg)
                })?;
            if re.is_match(&*t_str) {
                Ok(())
            } else {
                let default_msg = format!("Assertion failed: '{}' does not match regex '{}'", t_str, p_str);
                let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
                Err(Error::RuntimeError(error_msg))
            }
        }
        _ => {
            let default_msg = "Assertion failed: both arguments must be strings";
            let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
            Err(Error::RuntimeError(error_msg))
        }
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
/// * `message` - Optional custom error message (as Lua string or nil)
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
/// assert.table_has_key(config, "name")                                -- passes
/// assert.table_has_key(config, "version")                            -- passes
/// assert.table_has_key(config, "missing", "Config missing key")      -- fails with custom message
/// ```
fn assert_table_has_key(_lua: &Lua, (table, key, message): (Value, Value, Option<Value>)) -> LuaResult<()> {
    let custom_msg = message
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        });

    if let Value::Table(t) = table {
        if t.contains_key(key)? {
            Ok(())
        } else {
            let default_msg = "Assertion failed: table missing key";
            let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
            Err(Error::RuntimeError(error_msg))
        }
    } else {
        let default_msg = "Assertion failed: not a table";
        let error_msg = create_assertion_error(default_msg, custom_msg.as_deref());
        Err(Error::RuntimeError(error_msg))
    }
}

/// Asserts that a command/executable exists in the system PATH.
/// 
/// This function checks if a command is available by using the system's `which` 
/// (Unix/Linux/macOS) or `where` (Windows) command. This is essential for CI/CD 
/// scripts that depend on external tools and need to fail early if requirements 
/// are not met.
/// 
/// # Arguments
/// 
/// * `command` - The command name to check (as Lua string)
/// * `message` - Optional custom error message (as Lua string or nil)
/// 
/// # Returns
/// 
/// * `Ok(())` if the command exists and is accessible
/// * `Err(RuntimeError)` if the command is not found or cannot be executed
/// 
/// # Example
/// 
/// ```lua
/// assert.command_exists("git")                        -- passes if git is installed
/// assert.command_exists("docker", "Docker required")  -- fails with custom message if no docker
/// assert.command_exists("python3")                    -- passes if python3 is in PATH
/// assert.command_exists("nonexistent")                -- fails with default message
/// ```
fn assert_command_exists(_lua: &Lua, (command, message): (String, Option<Value>)) -> LuaResult<()> {
    let custom_msg = message
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        });

    // Use the appropriate command based on the operating system
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    
    let output = std::process::Command::new(which_cmd)
        .arg(&command)
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            // Command exists and was found
            Ok(())
        }
        Ok(_) => {
            // Command executed but failed (command not found)
            let default_msg = format!("Assertion failed: command '{}' not found in PATH", command);
            let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
            Err(Error::RuntimeError(error_msg))
        }
        Err(e) => {
            // Failed to execute which/where command
            let default_msg = format!("Assertion failed: unable to check command '{}': {}", command, e);
            let error_msg = create_assertion_error(&default_msg, custom_msg.as_deref());
            Err(Error::RuntimeError(error_msg))
        }
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

    #[test]
    fn test_assert_equals_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("equals").unwrap();
        
        let custom_msg = lua.create_string("Expected values to be equal").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(42), Value::Integer(43), Value::String(custom_msg)));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Expected values to be equal"));
        assert!(err_msg.contains("Assertion failed"));
    }

    #[test]
    fn test_assert_not_nil_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("not_nil").unwrap();
        
        let custom_msg = lua.create_string("User ID should not be nil").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Nil, Value::String(custom_msg)));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("User ID should not be nil"));
    }

    #[test]
    fn test_assert_is_true_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("is_true").unwrap();
        
        let custom_msg = lua.create_string("Build should have succeeded").unwrap();
        let result: mlua::Result<()> = f.call::<()>((false, Value::String(custom_msg)));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Build should have succeeded"));
    }

    #[test]
    fn test_assert_str_contains_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_contains").unwrap();
        
        let custom_msg = lua.create_string("Log should contain success message").unwrap();
        let result: mlua::Result<()> = f.call::<()>((
            Value::String(lua.create_string("Error: Failed to connect").unwrap()),
            Value::String(lua.create_string("SUCCESS").unwrap()),
            Value::String(custom_msg)
        ));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Log should contain success message"));
    }

    #[test]
    fn test_assert_str_matches_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("str_matches").unwrap();
        
        let custom_msg = lua.create_string("Version should match semver format").unwrap();
        let result: mlua::Result<()> = f.call::<()>((
            Value::String(lua.create_string("invalid_version").unwrap()),
            Value::String(lua.create_string(r"\d+\.\d+\.\d+").unwrap()),
            Value::String(custom_msg)
        ));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Version should match semver format"));
    }

    #[test]
    fn test_assert_table_has_key_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("table_has_key").unwrap();
        
        let table = lua.create_table().unwrap();
        table.set("name", "test").unwrap();
        
        let custom_msg = lua.create_string("Config must contain database_url").unwrap();
        let result: mlua::Result<()> = f.call::<()>((
            Value::Table(table),
            Value::String(lua.create_string("database_url").unwrap()),
            Value::String(custom_msg)
        ));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Config must contain database_url"));
    }

    #[test]
    fn test_assert_command_exists_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("command_exists").unwrap();
        
        // Test with a command that should exist on most systems
        let test_cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let result: mlua::Result<()> = f.call(test_cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_command_exists_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("command_exists").unwrap();
        
        // Test with a command that definitely doesn't exist
        let result: mlua::Result<()> = f.call("definitely_nonexistent_command_12345");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found in PATH"));
    }

    #[test]
    fn test_assert_command_exists_with_custom_message() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("command_exists").unwrap();
        
        let custom_msg = lua.create_string("Docker is required for containerized builds").unwrap();
        let result: mlua::Result<()> = f.call::<()>((
            "definitely_nonexistent_command_12345",
            Value::String(custom_msg)
        ));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Docker is required for containerized builds"));
    }
}
