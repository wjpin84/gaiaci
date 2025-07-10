//! # File System Module
//! 
//! This module provides comprehensive file system operations for Lua scripts in GaiaCI.
//! It enables CI pipelines to manipulate files and directories, manage build artifacts,
//! process configuration files, and organize output data with robust error handling.
//! 
//! ## Available Functions
//! 
//! - `read(path)` - Read the complete contents of a file as a string
//! - `write(path, content)` - Write content to a file, creating or overwriting as needed
//! - `exists(path)` - Check if a file or directory exists at the given path
//! - `is_dir(path)` - Determine if a path points to a directory
//! - `list_dir(path)` - List all entries in a directory as a Lua table
//! - `remove(path)` - Remove a file or recursively delete a directory
//! - `copy(src, dst)` - Copy a file from source to destination path
//! 
//! ## Error Handling
//! 
//! All functions properly handle system errors and propagate them to Lua as exceptions.
//! This ensures that file system operations integrate seamlessly with Lua's error
//! handling mechanisms and provide meaningful error messages for debugging.
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Basic file operations
//! fs.write("config.txt", "debug=true\nverbose=false")
//! local config = fs.read("config.txt")
//! print("Config:", config)
//! 
//! -- Directory operations
//! if fs.exists("build") and fs.is_dir("build") then
//!     local files = fs.list_dir("build")
//!     for i, file in ipairs(files) do
//!         print("Build artifact:", file)
//!     end
//! end
//! 
//! -- File management
//! fs.copy("template.conf", "production.conf")
//! fs.remove("temp_file.txt")
//! 
//! -- CI pipeline example
//! local test_results = fs.read("test-results.xml")
//! if fs.exists("artifacts") then
//!     fs.copy("build/app.jar", "artifacts/app-v1.0.jar")
//! end
//! ```

use mlua::{Lua, Result as LuaResult, Table};
use std::fs;
use std::path::Path;
use super::GaiaModule;

/// The File System module provides file and directory operations for Lua scripts.
/// 
/// This struct implements the GaiaModule trait to expose comprehensive file system
/// functions to Lua environments, enabling scripts to manipulate files, manage
/// directories, and handle build artifacts in CI pipeline workflows.
pub struct Fs;

impl GaiaModule for Fs {
    /// Creates a new File System module table with all file operation functions.
    /// 
    /// This method registers all available file system functions with the Lua environment,
    /// making them accessible under the `fs` namespace for comprehensive file and
    /// directory manipulation in CI pipeline scripts.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context to create the module in
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing all file system functions
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

/// Reads the complete contents of a file as a string.
/// 
/// This function reads the entire file at the specified path and returns its
/// contents as a UTF-8 string. It's useful for reading configuration files,
/// processing build outputs, or loading template files in CI pipelines.
/// 
/// # Arguments
/// 
/// * `path` - The file path to read from
/// 
/// # Returns
/// 
/// * `Ok(String)` containing the file contents if successful
/// * `Err(LuaError)` if the file doesn't exist, cannot be read, or contains invalid UTF-8
/// 
/// # Example
/// 
/// ```lua
/// local config = fs.read("config.json")
/// local readme = fs.read("README.md")
/// local build_log = fs.read("build.log")
/// ```
fn read_file(_: &Lua, path: String) -> LuaResult<String> {
    Ok(fs::read_to_string(&path)?)
}

/// Writes content to a file, creating or overwriting as necessary.
/// 
/// This function writes the provided string content to the specified file path.
/// If the file doesn't exist, it will be created. If it does exist, it will be
/// completely overwritten. Parent directories are not created automatically.
/// 
/// # Arguments
/// 
/// * `path` - The file path to write to
/// * `content` - The string content to write to the file
/// 
/// # Returns
/// 
/// * `Ok(())` if the write operation was successful
/// * `Err(LuaError)` if the file cannot be created or written to
/// 
/// # Example
/// 
/// ```lua
/// fs.write("output.txt", "Build completed successfully")
/// fs.write("config.json", '{"debug": true, "version": "1.0"}')
/// fs.write("deploy.sh", "#!/bin/bash\necho 'Deploying application'")
/// ```
fn write_file(_: &Lua, (path, content): (String, String)) -> LuaResult<()> {
    fs::write(&path, content)?;
    Ok(())
}

/// Checks if a file or directory exists at the specified path.
/// 
/// This function tests whether a file or directory exists at the given path
/// without attempting to access its contents. It's useful for conditional
/// operations and validating that required files are present before processing.
/// 
/// # Arguments
/// 
/// * `path` - The file or directory path to check
/// 
/// # Returns
/// 
/// * `true` if the path exists (as either a file or directory)
/// * `false` if the path does not exist
/// 
/// # Example
/// 
/// ```lua
/// if fs.exists("config.json") then
///     local config = fs.read("config.json")
/// else
///     print("Configuration file not found")
/// end
/// 
/// if fs.exists("build") then
///     print("Build directory exists")
/// end
/// ```
fn exists(_: &Lua, path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).exists())
}

/// Determines if the specified path points to a directory.
/// 
/// This function checks whether the given path is a directory (as opposed to
/// a regular file). It returns false if the path doesn't exist or points to
/// a file. This is essential for directory-specific operations and validation.
/// 
/// # Arguments
/// 
/// * `path` - The path to examine
/// 
/// # Returns
/// 
/// * `true` if the path exists and is a directory
/// * `false` if the path doesn't exist, is a file, or is another type of file system entry
/// 
/// # Example
/// 
/// ```lua
/// if fs.is_dir("src") then
///     local files = fs.list_dir("src")
///     print("Source directory contains", #files, "items")
/// else
///     print("src is not a directory")
/// end
/// ```
fn is_dir(_: &Lua, path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_dir())
}

/// Lists all entries in a directory as a Lua table.
/// 
/// This function reads the contents of a directory and returns all entries
/// (files, subdirectories, and other file system objects) as a numerically
/// indexed Lua table. The entries are returned as filename strings without
/// their full paths.
/// 
/// # Arguments
/// 
/// * `path` - The directory path to list
/// 
/// # Returns
/// 
/// * `Ok(Table)` containing directory entries as numbered indices (1, 2, 3, ...)
/// * `Err(LuaError)` if the path doesn't exist, isn't a directory, or can't be read
/// 
/// # Example
/// 
/// ```lua
/// local files = fs.list_dir("build")
/// for i, filename in ipairs(files) do
///     print("Found:", filename)
///     if filename:match("%.jar$") then
///         print("  -> Java archive file")
///     end
/// end
/// ```
fn list_dir(lua: &Lua, path: String) -> LuaResult<Table> {
    let result = lua.create_table()?;
    for (i, entry) in fs::read_dir(path)?.enumerate() {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        result.set(i + 1, name)?;
    }
    Ok(result)
}

/// Removes a file or recursively deletes a directory.
/// 
/// This function deletes the specified file or directory. For files, it performs
/// a simple deletion. For directories, it recursively removes all contents and
/// then the directory itself. Use with caution as this operation is irreversible.
/// 
/// # Arguments
/// 
/// * `path` - The file or directory path to remove
/// 
/// # Returns
/// 
/// * `Ok(())` if the removal was successful
/// * `Err(LuaError)` if the path doesn't exist or cannot be removed due to permissions
/// 
/// # Example
/// 
/// ```lua
/// -- Remove individual files
/// fs.remove("temp.log")
/// fs.remove("old_config.json")
/// 
/// -- Remove entire directories (be careful!)
/// fs.remove("temp_build")
/// fs.remove("old_artifacts")
/// ```
fn remove_path(_: &Lua, path: String) -> LuaResult<()> {
    let path = Path::new(&path);
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Copies a file from source to destination path.
/// 
/// This function creates an exact copy of the source file at the destination path.
/// If the destination file already exists, it will be overwritten. The destination
/// directory must already exist; this function does not create parent directories.
/// Only works with regular files, not directories.
/// 
/// # Arguments
/// 
/// * `src` - The source file path to copy from
/// * `dst` - The destination file path to copy to
/// 
/// # Returns
/// 
/// * `Ok(())` if the copy operation was successful
/// * `Err(LuaError)` if the source doesn't exist, destination can't be written, or other I/O errors
/// 
/// # Example
/// 
/// ```lua
/// -- Backup configuration files
/// fs.copy("config.json", "config.json.backup")
/// 
/// -- Deploy build artifacts
/// fs.copy("build/app.jar", "deploy/app.jar")
/// fs.copy("templates/nginx.conf", "/etc/nginx/sites-available/myapp")
/// 
/// -- Create versioned copies
/// fs.copy("release.zip", "releases/v1.0.0.zip")
/// ```
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
