//! # Shell Module
//! 
//! This module provides shell command execution capabilities for Lua scripts in GaiaCI.
//! It enables CI pipelines to run system commands, build tools, test suites, and deployment
//! scripts with full control over execution environment and output capture.
//! 
//! ## Available Functions
//! 
//! - `run(options)` - Execute a shell command with comprehensive configuration options
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
//! ```

use mlua::{Lua, Table, Value, Result as LuaResult};
use std::process::{Command, Stdio};
use super::GaiaModule;

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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_shell_module_creation() {
        let lua = Lua::new();
        let shell_mod = Shell::create(&lua).unwrap();
        
        // Test that the run function is present
        assert!(shell_mod.get::<mlua::Function>("run").is_ok());
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
}