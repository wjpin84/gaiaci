// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for file system operations.
//
// The fs module provides the following functions to Lua scripts:
// - read(path): Read the contents of a file as a string
// - write(path, content): Write content to a file
// - exists(path): Check if a file or directory exists
// - is_dir(path): Check if a path is a directory
// - list_dir(path): List the contents of a directory as a table
// - remove(path): Remove a file or directory (recursively for directories)
// - copy(src, dst): Copy a file from source to destination
//
// All functions properly handle errors and propagate them to Lua as exceptions.
// This module is essential for CI pipeline scripts that need to manipulate files,
// create build artifacts, manage temporary files, and organize output directories.
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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_fs_module_creation() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        
        // Test that all fs functions are present
        assert!(fs_mod.get::<mlua::Function>("read").is_ok());
        assert!(fs_mod.get::<mlua::Function>("write").is_ok());
        assert!(fs_mod.get::<mlua::Function>("exists").is_ok());
        assert!(fs_mod.get::<mlua::Function>("is_dir").is_ok());
        assert!(fs_mod.get::<mlua::Function>("list_dir").is_ok());
        assert!(fs_mod.get::<mlua::Function>("remove").is_ok());
        assert!(fs_mod.get::<mlua::Function>("copy").is_ok());
    }

    #[test]
    fn test_exists_function() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let exists_fn: mlua::Function = fs_mod.get("exists").unwrap();
        
        // Test with a path that should exist (current directory)
        let result: bool = exists_fn.call(".").unwrap();
        assert!(result);
        
        // Test with a path that shouldn't exist
        let result: bool = exists_fn.call("/nonexistent/path/12345").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_is_dir_function() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let is_dir_fn: mlua::Function = fs_mod.get("is_dir").unwrap();
        
        // Test with current directory
        let result: bool = is_dir_fn.call(".").unwrap();
        assert!(result);
        
        // Test with a file (if Cargo.toml exists)
        if std::path::Path::new("Cargo.toml").exists() {
            let result: bool = is_dir_fn.call("Cargo.toml").unwrap();
            assert!(!result);
        }
    }

    #[test]
    fn test_write_and_read_functions() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let write_fn: mlua::Function = fs_mod.get("write").unwrap();
        let read_fn: mlua::Function = fs_mod.get("read").unwrap();
        
        let test_file = "test_fs_unit.txt";
        let test_content = "Unit test content for fs module";
        
        // Write content to file
        let _: () = write_fn.call((test_file, test_content)).unwrap();
        
        // Read content back
        let read_content: String = read_fn.call(test_file).unwrap();
        assert_eq!(read_content, test_content);
        
        // Clean up
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_copy_function() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let write_fn: mlua::Function = fs_mod.get("write").unwrap();
        let copy_fn: mlua::Function = fs_mod.get("copy").unwrap();
        let read_fn: mlua::Function = fs_mod.get("read").unwrap();
        
        let source_file = "test_source_unit.txt";
        let dest_file = "test_dest_unit.txt";
        let test_content = "Content to be copied";
        
        // Create source file
        let _: () = write_fn.call((source_file, test_content)).unwrap();
        
        // Copy file
        let _: () = copy_fn.call((source_file, dest_file)).unwrap();
        
        // Verify copy
        let copied_content: String = read_fn.call(dest_file).unwrap();
        assert_eq!(copied_content, test_content);
        
        // Clean up
        let _ = fs::remove_file(source_file);
        let _ = fs::remove_file(dest_file);
    }

    #[test]
    fn test_list_dir_function() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let list_dir_fn: mlua::Function = fs_mod.get("list_dir").unwrap();
        
        // Test listing current directory (should have at least Cargo.toml and src/)
        let result: mlua::Table = list_dir_fn.call(".").unwrap();
        
        // Convert to a Vec to check contents
        let mut entries = Vec::new();
        for i in 1..=result.len().unwrap_or(0) {
            if let Ok(entry) = result.get::<String>(i) {
                entries.push(entry);
            }
        }
        
        // Should contain some common project files
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_remove_function() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let write_fn: mlua::Function = fs_mod.get("write").unwrap();
        let exists_fn: mlua::Function = fs_mod.get("exists").unwrap();
        let remove_fn: mlua::Function = fs_mod.get("remove").unwrap();
        
        let test_file = "test_remove_unit.txt";
        let test_content = "This file will be removed";
        
        // Create file
        let _: () = write_fn.call((test_file, test_content)).unwrap();
        
        // Verify it exists
        let exists_before: bool = exists_fn.call(test_file).unwrap();
        assert!(exists_before);
        
        // Remove file
        let _: () = remove_fn.call(test_file).unwrap();
        
        // Verify it's gone
        let exists_after: bool = exists_fn.call(test_file).unwrap();
        assert!(!exists_after);
    }

    #[test]
    fn test_read_nonexistent_file_error() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let read_fn: mlua::Function = fs_mod.get("read").unwrap();
        
        // Try to read a file that doesn't exist
        let result: mlua::Result<String> = read_fn.call("/nonexistent/file/path.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_invalid_path_error() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let write_fn: mlua::Function = fs_mod.get("write").unwrap();
        
        // Try to write to an invalid path
        let result: mlua::Result<()> = write_fn.call((
            "/invalid/nonexistent/path/file.txt",
            "content"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_dir_nonexistent_error() {
        let lua = Lua::new();
        let fs_mod = Fs::create(&lua).unwrap();
        let list_dir_fn: mlua::Function = fs_mod.get("list_dir").unwrap();
        
        // Try to list a directory that doesn't exist
        let result: mlua::Result<mlua::Table> = list_dir_fn.call("/nonexistent/directory");
        assert!(result.is_err());
    }
}
