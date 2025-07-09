// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for shell command execution.
//
// The shell module provides the following function to Lua scripts:
// - run(options): Execute a shell command with various options
//
// Options table can contain:
// - cmd (required): The command string to execute
// - capture (optional, default false): Whether to capture stdout/stderr
// - env (optional): Table of environment variables to set
// - cwd (optional): Working directory to execute the command in
// - shell (optional): Custom shell to use (defaults to /bin/sh on Unix, cmd on Windows)
//
// Returns a table with:
// - success: Boolean indicating if command succeeded
// - code: Exit code of the command
// - stdout: Captured stdout (only if capture=true)
// - stderr: Captured stderr (only if capture=true)
//
// This module is essential for CI pipeline scripts that need to run build commands,
// execute tests, deploy applications, and interact with the system.
use mlua::{Lua, Table, Value, Result as LuaResult};
use std::process::{Command, Stdio};
use super::GaiaModule;

pub struct Shell;

// Helper function to build the shell command
impl GaiaModule for Shell {
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

// Build the shell command based on the operating system
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

// Apply the current working directory and environment variables to the command
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

// Run the command without capturing output
fn run_without_capture(lua: &Lua, command: &mut Command) -> LuaResult<Value> {
    let status = command.status()?;

    let result = lua.create_table()?;
    result.set("code", status.code().unwrap_or(-1))?;
    result.set("success", status.success())?;
    Ok(Value::Table(result))
}

// Run the command and capture its output
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