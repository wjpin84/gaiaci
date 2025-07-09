// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for assertion functions.
use mlua::{Lua, Result as LuaResult, Table, Value, Error};
use super::GaiaModule;
use regex;

pub struct Assert;

impl GaiaModule for Assert {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let tbl = lua.create_table()?;
        tbl.set("equals", lua.create_function(assert_equals)?)?;
        tbl.set("not_nil", lua.create_function(assert_not_nil)?)?;
        tbl.set("nil", lua.create_function(assert_nil)?)?;
        tbl.set("is_true", lua.create_function(assert_true)?)?;
        tbl.set("str_contains", lua.create_function(assert_str_contains)?)?;
        tbl.set("str_matches", lua.create_function(assert_str_matches)?)?;
        tbl.set("table_has_key", lua.create_function(assert_table_has_key)?)?;
        Ok(tbl)
    }
}

// Assert equals function for Lua
fn assert_equals(_: &Lua, (a, b): (Value, Value)) -> LuaResult<()> {
    if a != b {
        Err(Error::RuntimeError(format!("Assertion failed: {:?} != {:?}", a, b)))
    } else {
        Ok(())
    }
}

// Assert not nil function for Lua
fn assert_not_nil(_: &Lua, val: Value) -> LuaResult<()> {
    if matches!(val, Value::Nil) {
        Err(Error::RuntimeError("Assertion failed: value is nil".to_string()))
    } else {
        Ok(())
    }
}

fn assert_nil(_: &Lua, val: Value) -> LuaResult<()> {
    if !matches!(val, Value::Nil) {
        Err(Error::RuntimeError("Assertion failed: value is not nil".to_string()))
    } else {
        Ok(())
    }
}

// Assert true function for Lua
fn assert_true(_: &Lua, cond: bool) -> LuaResult<()> {
    if !cond {
        Err(Error::RuntimeError("Assertion failed: condition is false".to_string()))
    } else {
        Ok(())
    }
}

// Assert string contains function for Lua
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

// Assert string matches regex pattern function for Lua
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

// Assert table has key function for Lua
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
    fn test_assert_equals_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("equals").unwrap();
        f.call::<()>((Value::Integer(42), Value::Integer(42))).unwrap();
    }

    #[test]
    fn test_assert_equals_fail() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("equals").unwrap();
        let result: mlua::Result<()> = f.call::<()>((Value::Integer(42), Value::Integer(43)));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Assertion failed"));
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
        let f: mlua::Function = assert.get("nil").unwrap();
        let result: mlua::Result<()> = f.call::<()>(Value::String(lua.create_string("hello").unwrap()));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("value is not nil"));
    }

    #[test]
    fn test_assert_nil_pass() {
        let lua = Lua::new();
        let assert = Assert::create(&lua).unwrap();
        let f: mlua::Function = assert.get("nil").unwrap();
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
