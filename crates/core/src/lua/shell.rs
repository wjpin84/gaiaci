//! # Shell Module
//! 
//! This module provides shell command execution capabilities for Lua scripts in GaiaCI.
//! It enables CI pipelines to run system commands, build tools, test suites, and deployment
//! scripts with full control over execution environment and output capture.
//! 
//! ## Available Functions
//! 
//! - `run(options)` - Execute a shell command with comprehensive configuration options
//! - `pwd()` - Get the current working directory (cross-platform)
//! - `shell()` - Get the current shell name (bash, zsh, cmd, pwsh, etc.)
//! - `os()` - Get the operating system name (linux, windows, macos)
//! - `arch()` - Get the system architecture (x86_64, aarch64, etc.)
//! - `which(command)` - Find the full path to an executable command
//! 
//! ## Command Options
//! 
//! The `run` function accepts a table with the following options:
//! 
//! - **cmd** (required) - The command string to execute
//! - **capture** (optional, default: false) - Whether to capture stdout/stderr output
//! - **env** (optional) - Table of environment variables to set for the command
//! - **cwd** (optional) - Working directory to execute the command in
//! - **shell** (optional) - Custom shell to use (defaults to `/bin/sh` on Unix, `cmd` on Windows)
//! 
//! ## Return Value
//! 
//! Returns a table containing:
//! 
//! - **success** - Boolean indicating if the command succeeded (exit code 0)
//! - **code** - Integer exit code of the command
//! - **stdout** - Captured stdout as string (only if `capture=true`)
//! - **stderr** - Captured stderr as string (only if `capture=true`)
//! 
//! ## Example Usage in Lua
//! 
//! ```lua
//! -- Simple command execution
//! local result = shell.run({cmd = "echo hello"})
//! print("Success:", result.success)
//! print("Exit code:", result.code)
//! 
//! -- Capture output
//! local result = shell.run({
//!     cmd = "ls -la",
//!     capture = true
//! })
//! print("Files:", result.stdout)
//! 
//! -- Run with custom environment and working directory
//! local result = shell.run({
//!     cmd = "npm test",
//!     capture = true,
//!     cwd = "/path/to/project",
//!     env = {
//!         NODE_ENV = "test",
//!         CI = "true"
//!     }
//! })
//! 
//! -- Use custom shell
//! local result = shell.run({
//!     cmd = "echo $0",
//!     capture = true,
//!     shell = "/bin/bash"
//! })
//! 
//! -- Environment detection and platform-specific commands
//! print("Current directory:", shell.pwd())
//! print("Shell:", shell.shell())
//! print("OS:", shell.os())
//! print("Architecture:", shell.arch())
//! 
//! -- Check for required tools
//! local function has_tool(name)
//!     local success, _ = pcall(function() return shell.which(name) end)
//!     return success
//! end
//! 
//! -- Or use the simpler exists function
//! if shell.exists("docker") then
//!     shell.run({cmd = "docker build -t myapp ."})
//! else
//!     error("Docker is required but not found")
//! end
//! 
//! -- Check absolute paths
//! if shell.exists("/usr/bin/python3") then
//!     print("Python3 available at system location")
//! end
//! 
//! -- Batch check for CI dependencies
//! local required = {"git", "curl", "tar"}
//! local missing = {}
//! for _, tool in ipairs(required) do
//!     if not shell.exists(tool) then
//!         table.insert(missing, tool)
//!     end
//! end
//! if #missing > 0 then
//!     error("Missing tools: " .. table.concat(missing, ", "))
//! end
//! 
//! -- Platform-specific CI commands
//! local os_name = shell.os()
//! if os_name == "linux" then
//!     shell.run({cmd = "sudo apt-get update"})
//! elseif os_name == "macos" then
//!     shell.run({cmd = "brew install dependencies"})
//! elseif os_name == "windows" then
//!     shell.run({cmd = "choco install dependencies"})
//! end
//! ```

use mlua::{Lua, Table, Value, Result as LuaResult};
use std::process::{Command, Stdio};
use crate::lua::common::GaiaModule;

/// The Shell module provides command execution functionality for Lua scripts.
/// 
/// This struct implements the GaiaModule trait to expose shell command execution
/// functions to Lua environments, enabling scripts to interact with the system,
/// run build tools, execute tests, and perform deployment operations.
pub struct Shell;

impl GaiaModule for Shell {
    /// Creates a new Shell module table with command execution functions.
    /// 
    /// This method registers the shell command execution function with the Lua environment,
    /// making it accessible under the `shell` namespace for running system commands.
    /// 
    /// # Arguments
    /// 
    /// * `lua` - The Lua context to create the module in
    /// 
    /// # Returns
    /// 
    /// A `LuaResult<Table>` containing the shell execution functions
    fn create(lua: &Lua) -> LuaResult<Table> {
         let shell = lua.create_table()?;

        let run_fn = lua.create_function(move |lua, opts: Table| {
            let cmd: String = opts.get("cmd")?;
            let shell_path: Option<String> = opts.get("shell").ok();
            let env_table: Option<Table> = opts.get("env").ok();
            let cwd: Option<String> = opts.get("cwd").ok();
            let capture: bool = opts.get("capture").unwrap_or(false);

            let shell_to_use = shell_path.unwrap_or_else(|| {
                if cfg!(windows) { "cmd".to_string() } else { "/bin/sh".to_string() }
            });

            let mut command = build_shell_command(&cmd, &shell_to_use);
            apply_env_and_cwd(&mut command, cwd, env_table)?;

            if capture {
                run_with_capture(lua, &mut command)
            } else {
                run_without_capture(lua, &mut command)
            }
        })?;

        shell.set("run", run_fn)?;
        shell.set("pwd", lua.create_function(get_current_dir)?)?;
        shell.set("shell", lua.create_function(get_current_shell)?)?;
        shell.set("os", lua.create_function(get_os_info)?)?;
        shell.set("arch", lua.create_function(get_arch_info)?)?;
        shell.set("which", lua.create_function(which_command)?)?;
        Ok(shell)
    }

}

/// Builds a shell command for execution based on the operating system.
/// 
/// This function creates a `Command` instance configured for the target operating system.
/// On Unix-like systems, it uses the specified shell with the `-c` flag to execute
/// the command string. On Windows, it uses the specified shell (typically `cmd`)
/// with the `/C` flag.
/// 
/// # Arguments
/// 
/// * `cmd` - The command string to execute
/// * `shell` - The shell executable path to use
/// 
/// # Returns
/// 
/// A configured `Command` instance ready for execution
/// 
/// # Example
/// 
/// ```text
/// build_shell_command("echo hello", "/bin/bash");
/// // On Unix: /bin/bash -c "echo hello"
/// // On Windows: cmd /C "echo hello"
/// ```
fn build_shell_command(cmd: &str, shell: &str) -> Command {
      let command = if cfg!(windows) {
        let mut c = Command::new(shell);
        c.arg("/C").arg(cmd);
        c
    } else {
        let mut c = Command::new(shell);
        c.arg("-c").arg(cmd);
        c
    };
    command
}

/// Applies working directory and environment variables to a command.
/// 
/// This function configures a `Command` instance with optional working directory
/// and environment variables. Environment variables from the Lua table are
/// converted from Lua string values to system environment variables.
/// 
/// # Arguments
/// 
/// * `command` - The mutable command to configure
/// * `cwd` - Optional working directory path
/// * `envs` - Optional Lua table containing environment variable key-value pairs
/// 
/// # Returns
/// 
/// * `Ok(())` if configuration was successful
/// * `Err(LuaError)` if there were issues processing the environment table
/// 
/// # Example
/// 
/// ```lua
/// -- In Lua, this creates environment variables for the command
/// local env_vars = {
///     PATH = "/custom/bin:" .. os.getenv("PATH"),
///     NODE_ENV = "production"
/// }
/// ```
fn apply_env_and_cwd(command: &mut Command, cwd: Option<String>, envs: Option<Table>) -> LuaResult<()> {
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    if let Some(env_table) = envs {
        for pair in env_table.pairs::<Value, Value>() {
            let (k, v) = pair?;
            if let (Value::String(k), Value::String(v)) = (k, v) {
                let key = k.to_str()?.to_string();
                let val = v.to_str()?.to_string();
                command.env(key, val);
            }
        }
    }

    Ok(())
}

/// Executes a command without capturing its output.
/// 
/// This function runs the command and waits for it to complete, but does not
/// capture stdout or stderr. The output will be displayed directly to the
/// terminal. This is useful for interactive commands or when you want to
/// see real-time output during CI pipeline execution.
/// 
/// # Arguments
/// 
/// * `lua` - The Lua context for creating the result table
/// * `command` - The configured command to execute
/// 
/// # Returns
/// 
/// A Lua table containing:
/// - `success`: Boolean indicating if the command succeeded
/// - `code`: Integer exit code of the command
/// 
/// # Example
/// 
/// ```lua
/// -- Output goes directly to terminal
/// local result = shell.run({
///     cmd = "make build",
///     capture = false
/// })
/// print("Build succeeded:", result.success)
/// ```
fn run_without_capture(lua: &Lua, command: &mut Command) -> LuaResult<Value> {
    let status = command.status()?;

    let result = lua.create_table()?;
    result.set("code", status.code().unwrap_or(-1))?;
    result.set("success", status.success())?;
    Ok(Value::Table(result))
}

/// Executes a command and captures its output.
/// 
/// This function runs the command and captures both stdout and stderr streams,
/// returning them as strings in the result table. This is essential for
/// processing command output, parsing build results, or capturing error
/// messages for further analysis in CI pipelines.
/// 
/// # Arguments
/// 
/// * `lua` - The Lua context for creating the result table
/// * `command` - The configured command to execute with output capture
/// 
/// # Returns
/// 
/// A Lua table containing:
/// - `success`: Boolean indicating if the command succeeded
/// - `code`: Integer exit code of the command
/// - `stdout`: String containing the captured standard output
/// - `stderr`: String containing the captured standard error
/// 
/// # Example
/// 
/// ```lua
/// -- Capture and process output
/// local result = shell.run({
///     cmd = "npm test",
///     capture = true
/// })
/// 
/// if result.success then
///     print("Tests passed!")
///     print("Output:", result.stdout)
/// else
///     print("Tests failed:", result.stderr)
/// end
/// ```
fn run_with_capture(lua: &Lua, command: &mut Command) -> LuaResult<Value> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = command.output()?;

    let result = lua.create_table()?;
    result.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
    result.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
    result.set("code", output.status.code().unwrap_or(-1))?;
    result.set("success", output.status.success())?;
    Ok(Value::Table(result))
}

/// Gets the current working directory path.
/// 
/// This function returns the absolute path of the current working directory,
/// which is useful for CI scripts to understand their execution context
/// and construct relative paths to build artifacts, configuration files, etc.
/// 
/// # Returns
/// 
/// * `Ok(String)` containing the current working directory path
/// * `Err(LuaError)` if the current directory cannot be determined
/// 
/// # Example
/// 
/// ```lua
/// local current_dir = shell.pwd()
/// print("Working in:", current_dir)
/// 
/// -- Use it to construct paths
/// local config_path = current_dir .. "/config.json"
/// ```
fn get_current_dir(_: &Lua, _: ()) -> LuaResult<String> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.to_string_lossy().to_string())
}

/// Gets the name of the current shell.
/// 
/// This function determines the current shell being used by examining
/// environment variables and system configuration. It's useful for
/// CI scripts that need to adapt their behavior based on the shell environment.
/// 
/// # Returns
/// 
/// * The shell name as a string (bash, zsh, cmd, pwsh, sh, etc.)
/// * Returns "unknown" if the shell cannot be determined
/// 
/// # Example
/// 
/// ```lua
/// local current_shell = shell.shell()
/// print("Running in shell:", current_shell)
/// 
/// if current_shell == "bash" then
///     print("Using bash-specific features")
/// elseif current_shell == "pwsh" then
///     print("Running in PowerShell")
/// end
/// ```
fn get_current_shell(_: &Lua, _: ()) -> LuaResult<String> {
    // First try to get from SHELL environment variable (Unix-like systems)
    if let Ok(shell_path) = std::env::var("SHELL") {
        if let Some(shell_name) = std::path::Path::new(&shell_path).file_name() {
            return Ok(shell_name.to_string_lossy().to_string());
        }
    }
    
    // On Windows, check for PowerShell or Command Prompt
    if cfg!(windows) {
        // Check if we're in PowerShell
        if std::env::var("PSModulePath").is_ok() {
            return Ok("pwsh".to_string());
        }
        // Check if we're in Windows PowerShell
        if std::env::var("POWERSHELL_VERSION").is_ok() {
            return Ok("powershell".to_string());
        }
        // Default to cmd on Windows
        return Ok("cmd".to_string());
    }
    
    // Fallback: try to detect from parent process or default
    Ok("unknown".to_string())
}

/// Gets the operating system name.
/// 
/// This function returns a standardized name for the current operating system,
/// which is essential for CI pipelines that need to run different commands
/// or use different tools based on the target platform.
/// 
/// # Returns
/// 
/// * The OS name as a string: "linux", "windows", "macos", or "unknown"
/// 
/// # Example
/// 
/// ```lua
/// local os_name = shell.os()
/// print("Running on:", os_name)
/// 
/// if os_name == "linux" then
///     shell.run({cmd = "apt-get update"})
/// elseif os_name == "macos" then
///     shell.run({cmd = "brew update"})
/// elseif os_name == "windows" then
///     shell.run({cmd = "choco upgrade"})
/// end
/// ```
fn get_os_info(_: &Lua, _: ()) -> LuaResult<String> {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "freebsd") {
        "freebsd"
    } else if cfg!(target_os = "openbsd") {
        "openbsd"
    } else if cfg!(target_os = "netbsd") {
        "netbsd"
    } else {
        "unknown"
    };
    
    Ok(os.to_string())
}

/// Gets the system architecture.
/// 
/// This function returns the CPU architecture of the current system,
/// which is useful for CI pipelines that need to build or deploy
/// architecture-specific binaries and packages.
/// 
/// # Returns
/// 
/// * The architecture as a string: "x86_64", "aarch64", "x86", etc.
/// 
/// # Example
/// 
/// ```lua
/// local arch = shell.arch()
/// print("Architecture:", arch)
/// 
/// if arch == "aarch64" then
///     print("Running on ARM64")
/// elseif arch == "x86_64" then
///     print("Running on x64")
/// end
/// 
/// -- Use for selecting correct binary
/// local binary_name = "myapp-" .. shell.os() .. "-" .. arch
/// ```
fn get_arch_info(_: &Lua, _: ()) -> LuaResult<String> {
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else {
        "unknown"
    };
    
    Ok(arch.to_string())
}

/// Finds the full path to an executable command.
/// 
/// This function searches the system PATH to locate an executable command,
/// similar to the Unix `which` command or Windows `where` command. It's
/// useful for CI scripts to verify that required tools are available.
/// 
/// # Arguments
/// 
/// * `command` - The command name to search for
/// 
/// # Returns
/// 
/// * `Ok(String)` containing the full path to the executable if found
/// * `Err(LuaError)` if the command is not found in PATH
/// 
/// # Example
/// 
/// ```lua
/// -- Check if required tools are available
/// local function check_tool(name)
///     local success, path = pcall(function() return shell.which(name) end)
///     if success then
///         print(name .. " found at:", path)
///         return true
///     else
///         print("ERROR: " .. name .. " not found in PATH")
///         return false
///     end
/// end
/// 
/// check_tool("git")
/// check_tool("node")
/// check_tool("docker")
/// ```
fn which_command(_: &Lua, command: String) -> LuaResult<String> {
    use std::process::Command;
    
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    
    let output = Command::new(which_cmd)
        .arg(&command)
        .output()
        .map_err(|e| mlua::Error::external(format!("Failed to run {}: {}", which_cmd, e)))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let path = stdout.trim().lines().next().unwrap_or("").to_string();
        if path.is_empty() {
            return Err(mlua::Error::external(format!("Command '{}' not found", command)));
        }
        Ok(path)
    } else {
        Err(mlua::Error::external(format!("Command '{}' not found", command)))
    }
}

/// Checks if an executable exists by PATH lookup or absolute path.
/// 
/// This function checks whether an executable exists either in the system PATH
/// or at an absolute file path. It's more flexible than `which` as it returns
/// a boolean and handles both relative command names and absolute paths.
/// 
/// # Arguments
/// 
/// * `path_or_command` - Either a command name (searches PATH) or absolute path to check
/// 
/// # Returns
/// 
/// * `true` if the executable exists and is accessible
/// * `false` if the executable doesn't exist or isn't accessible
/// 
/// # Example
/// 
/// ```lua
/// -- Check commands in PATH
/// if shell.exists("git") then
///     print("Git is available")
/// end
/// 
/// if shell.exists("docker") then
///     shell.run({cmd = "docker --version", capture = true})
/// else
///     error("Docker is required but not available")
/// end
/// 
/// -- Check absolute paths
/// if shell.exists("/usr/bin/python3") then
///     print("Python3 found at system location")
/// end
/// 
/// -- Check relative paths (treats as command name)
/// if shell.exists("node") then
///     print("Node.js is in PATH")
/// end
/// 
/// -- Batch check for CI dependencies
/// local required_tools = {"git", "curl", "tar", "gzip"}
/// local missing = {}
/// for _, tool in ipairs(required_tools) do
///     if not shell.exists(tool) then
///         table.insert(missing, tool)
///     end
/// end
/// 
/// if #missing > 0 then
///     error("Missing required tools: " .. table.concat(missing, ", "))
/// end
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_shell_module_creation() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        
        // Test that all shell functions are present
        assert!(shell_mod.get::<mlua::Function>("run").is_ok());
        assert!(shell_mod.get::<mlua::Function>("pwd").is_ok());
        assert!(shell_mod.get::<mlua::Function>("shell").is_ok());
        assert!(shell_mod.get::<mlua::Function>("os").is_ok());
        assert!(shell_mod.get::<mlua::Function>("arch").is_ok());
        assert!(shell_mod.get::<mlua::Function>("which").is_ok());
    }

    #[test]
    fn test_build_shell_command_unix() {
        let cmd = "echo hello";
        let shell = "/bin/sh";
        let command = build_shell_command(cmd, shell);
        
        // On Unix systems, should use the specified shell
        let program = command.get_program();
        assert_eq!(program, "/bin/sh");
        
        // We can't easily test the args without making the function public or restructuring,
        // but we can test that it doesn't panic and creates a valid Command
    }

    #[test]
    fn test_build_shell_command_with_different_shells() {
        let cmd = "ls -la";
        
        // Test with bash
        let command1 = build_shell_command(cmd, "/bin/bash");
        assert_eq!(command1.get_program(), "/bin/bash");
        
        // Test with zsh
        let command2 = build_shell_command(cmd, "/bin/zsh");
        assert_eq!(command2.get_program(), "/bin/zsh");
    }

    #[test]
    fn test_apply_env_and_cwd_no_options() {
        let mut command = Command::new("echo");
        let result = apply_env_and_cwd(&mut command, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_env_and_cwd_with_cwd() {
        let mut command = Command::new("pwd");
        let result = apply_env_and_cwd(&mut command, Some("/tmp".to_string()), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_env_and_cwd_with_env_table() {
        let lua = Lua::new();
        let env_table = lua.create_table().unwrap();
        env_table.set("TEST_KEY", "test_value").unwrap();
        env_table.set("ANOTHER_KEY", "another_value").unwrap();
        
        let mut command = Command::new("env");
        let result = apply_env_and_cwd(&mut command, None, Some(env_table));
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_run_basic_command_without_capture() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "echo test").unwrap();
        opts.set("capture", false).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let code: i32 = result_table.get("code").unwrap();
            assert!(success);
            assert_eq!(code, 0);
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_basic_command_with_capture() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "echo hello_world").unwrap();
        opts.set("capture", true).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let stdout: String = result_table.get("stdout").unwrap();
            let stderr: String = result_table.get("stderr").unwrap();
            let code: i32 = result_table.get("code").unwrap();
            
            assert!(success);
            assert_eq!(code, 0);
            assert!(stdout.contains("hello_world"));
            assert!(stderr.is_empty() || stderr.trim().is_empty());
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_with_environment_variables() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let env_table = lua.create_table().unwrap();
        env_table.set("TEST_VAR", "unit_test_value").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "echo $TEST_VAR").unwrap();
        opts.set("capture", true).unwrap();
        opts.set("env", env_table).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let stdout: String = result_table.get("stdout").unwrap();
            
            assert!(success);
            assert!(stdout.contains("unit_test_value"));
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_with_working_directory() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "pwd").unwrap();
        opts.set("capture", true).unwrap();
        opts.set("cwd", "/tmp").unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let stdout: String = result_table.get("stdout").unwrap();
            
            assert!(success);
            assert!(stdout.trim().ends_with("tmp") || stdout.contains("/tmp"));
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_failing_command() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "false").unwrap(); // Command that always fails
        opts.set("capture", true).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let code: i32 = result_table.get("code").unwrap();
            
            assert!(!success);
            assert_ne!(code, 0);
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_command_with_stderr() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "echo 'error message' >&2").unwrap(); // Write to stderr
        opts.set("capture", true).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let stderr: String = result_table.get("stderr").unwrap();
            assert!(stderr.contains("error message"));
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_with_custom_shell() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        opts.set("cmd", "echo custom_shell_test").unwrap();
        opts.set("capture", true).unwrap();
        opts.set("shell", "/bin/bash").unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_ok());
        
        if let mlua::Value::Table(result_table) = result.unwrap() {
            let success: bool = result_table.get("success").unwrap();
            let stdout: String = result_table.get("stdout").unwrap();
            
            assert!(success);
            assert!(stdout.contains("custom_shell_test"));
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_shell_run_missing_cmd_error() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let run_fn: mlua::Function = shell_mod.get("run").unwrap();
        
        let opts = lua.create_table().unwrap();
        // Intentionally not setting "cmd"
        opts.set("capture", true).unwrap();
        
        let result = run_fn.call::<mlua::Value>(opts);
        assert!(result.is_err()); // Should fail because cmd is required
    }

    #[test]
    fn test_shell_pwd_function() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let pwd_fn: mlua::Function = shell_mod.get("pwd").unwrap();
        
        let result: String = pwd_fn.call(()).unwrap();
        
        // Should return a non-empty path
        assert!(!result.is_empty());
        
        // Should be an absolute path (starts with / on Unix or drive letter on Windows)
        assert!(result.starts_with('/') || (result.len() >= 3 && result.chars().nth(1) == Some(':')));
    }

    #[test]
    fn test_shell_shell_function() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let shell_fn: mlua::Function = shell_mod.get("shell").unwrap();
        
        let result: String = shell_fn.call(()).unwrap();
        
        // Should return a non-empty shell name
        assert!(!result.is_empty());
        
        // Should be one of the known shell types or "unknown"
        let known_shells = vec!["bash", "zsh", "sh", "fish", "cmd", "pwsh", "powershell", "unknown"];
        assert!(known_shells.contains(&result.as_str()), "Unexpected shell: {}", result);
    }

    #[test]
    fn test_shell_os_function() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let os_fn: mlua::Function = shell_mod.get("os").unwrap();
        
        let result: String = os_fn.call(()).unwrap();
        
        // Should return one of the known OS types
        let known_os = vec!["linux", "windows", "macos", "freebsd", "openbsd", "netbsd", "unknown"];
        assert!(known_os.contains(&result.as_str()), "Unexpected OS: {}", result);
        
        // Test that it matches the compile-time target
        if cfg!(target_os = "linux") {
            assert_eq!(result, "linux");
        } else if cfg!(target_os = "windows") {
            assert_eq!(result, "windows");
        } else if cfg!(target_os = "macos") {
            assert_eq!(result, "macos");
        }
    }

    #[test]
    fn test_shell_arch_function() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let arch_fn: mlua::Function = shell_mod.get("arch").unwrap();
        
        let result: String = arch_fn.call(()).unwrap();
        
        // Should return one of the known architectures
        let known_archs = vec!["x86_64", "aarch64", "x86", "arm", "riscv64", "unknown"];
        assert!(known_archs.contains(&result.as_str()), "Unexpected architecture: {}", result);
        
        // Test that it matches the compile-time target
        if cfg!(target_arch = "x86_64") {
            assert_eq!(result, "x86_64");
        } else if cfg!(target_arch = "aarch64") {
            assert_eq!(result, "aarch64");
        }
    }

    #[test]
    fn test_shell_which_function() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let which_fn: mlua::Function = shell_mod.get("which").unwrap();
        
        // Test with a command that should exist on most systems
        let test_cmd = if cfg!(windows) { "cmd" } else { "sh" };
        let result: mlua::Result<String> = which_fn.call(test_cmd);
        
        // Should find the command
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.is_empty());
        assert!(path.contains(test_cmd));
    }

    #[test]
    fn test_shell_which_nonexistent_command() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        let which_fn: mlua::Function = shell_mod.get("which").unwrap();
        
        // Test with a command that definitely doesn't exist
        let result: mlua::Result<String> = which_fn.call("definitely_nonexistent_command_12345");
        
        // Should fail to find the command
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_module_has_all_functions() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        
        // Test that all expected functions are present
        assert!(shell_mod.get::<mlua::Function>("run").is_ok());
        assert!(shell_mod.get::<mlua::Function>("pwd").is_ok());
        assert!(shell_mod.get::<mlua::Function>("shell").is_ok());
        assert!(shell_mod.get::<mlua::Function>("os").is_ok());
        assert!(shell_mod.get::<mlua::Function>("arch").is_ok());
        assert!(shell_mod.get::<mlua::Function>("which").is_ok());
    }
}