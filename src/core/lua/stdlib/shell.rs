// This file is part of GaiaCI, a continuous integration system.
// It provides Lua bindings for shell command execution.
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