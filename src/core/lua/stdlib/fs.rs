// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for file system operations.
use mlua::{Lua, Result as LuaResult, Table};
use std::fs;
use std::path::Path;
use super::GaiaModule;

pub struct Fs;

impl GaiaModule for Fs {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let fs_table = lua.create_table()?;

        fs_table.set("read", lua.create_function(read_file)?)?;
        fs_table.set("write", lua.create_function(write_file)?)?;
        fs_table.set("exists", lua.create_function(exists)?)?;
        fs_table.set("is_dir", lua.create_function(is_dir)?)?;
        fs_table.set("list_dir", lua.create_function(list_dir)?)?;
        fs_table.set("remove", lua.create_function(remove_path)?)?;
        fs_table.set("copy", lua.create_function(copy_path)?)?;

        Ok(fs_table)
    }
}

// Read operation on a file
fn read_file(_: &Lua, path: String) -> LuaResult<String> {
    Ok(fs::read_to_string(&path)?)
}

// Write operation to a file
fn write_file(_: &Lua, (path, content): (String, String)) -> LuaResult<()> {
    fs::write(&path, content)?;
    Ok(())
}

// Check if a file or directory exists
fn exists(_: &Lua, path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).exists())
}

// Check if a path is a directory
fn is_dir(_: &Lua, path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_dir())
}

// List the contents of a directory
fn list_dir(lua: &Lua, path: String) -> LuaResult<Table> {
    let result = lua.create_table()?;
    for (i, entry) in fs::read_dir(path)?.enumerate() {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        result.set(i + 1, name)?;
    }
    Ok(result)
}

// Remove a file or directory
fn remove_path(_: &Lua, path: String) -> LuaResult<()> {
    let path = Path::new(&path);
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

// Copy a file from source to destination
fn copy_path(_: &Lua, (src, dst): (String, String)) -> LuaResult<()> {
    fs::copy(&src, &dst)?;
    Ok(())
}
