use mlua::{Lua, Table, Value};
use std::process::Command;
use tracing::{info, error};
use std::collections::HashMap;


/// Runs the entire pipeline by iterating over all jobs defined in the pipeline table.
/// 
/// # Arguments
/// * `lua` - The Lua context.
/// * `pipeline` - The Lua table representing the pipeline configuration.
/// 
/// # Errors
/// Returns an error if any job fails to execute.
pub fn run_pipeline(lua: &Lua, pipeline: Table) -> Result<(), Box<dyn std::error::Error>> {
    let jobs: Table = pipeline.get("jobs")?;
    let env_table: Option<Table> = pipeline.get("env").ok();
    let mut runtime_env = std::env::vars().collect::<std::collections::HashMap<_, _>>();

    if let Some(env) = env_table {
        for pair in env.pairs::<Value, Value>() {
            let (k, v) = pair?;
            if let (Value::String(k), Value::String(v)) = (k, v) {
                runtime_env.insert(k.to_str()?.to_string(), v.to_str()?.to_string());
            }
        }
    }

    for pair in jobs.pairs::<Value, Value>() {
        let (name, job_value) = pair?;
        let job: Table = match job_value {
            Value::Table(t) => t,
            _ => continue,
        };

        let name = match name {
            Value::String(s) => s.to_str()?.to_string(),
            _ => continue,
        };

        info!("▶ Running job: {}", name);
        run_job(lua, &name, &job, &runtime_env)?;
    }
    Ok(())
}

/// Executes a single job, supporting both job-level `run` (string or function)
/// and a `steps` array of step tables.
/// 
/// # Arguments
/// * `lua` - The Lua context.
/// * `job_name` - The name of the job.
/// * `job` - The Lua table representing the job.
/// * `runtime_env` - The environment variables to use for the job.
/// 
/// # Errors
/// Returns an error if any step or command fails to execute.
fn run_job(
    lua: &Lua,
    job_name: &str,
    job: &Table,
    runtime_env: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Support steps array
    if let Ok(steps) = job.get::<Table>("steps") {
        for step in steps.sequence_values::<Table>() {
            let step = step?;
            run_step(lua, job_name, &step, runtime_env)?;
        }
        return Ok(());
    }

    // Job-level run function
    if let Ok(func) = job.get::<mlua::Function>("run") {
        let env_table = lua.create_table()?;
        for (k, v) in runtime_env {
            env_table.set(k.as_str(), v.as_str())?;
        }
        func.call::<Table>(env_table)?;
        return Ok(());
    }

    // Job-level run string
    if let Ok(cmd) = job.get::<mlua::String>("run") {
        run_command(job_name, &cmd.to_str()?, runtime_env)?;
        return Ok(());
    }

    eprintln!("⚠️ Unsupported job type for '{}'", job_name);
    Ok(())
}

/// Executes a single step within a job, supporting both string and function `run` types.
/// 
/// # Arguments
/// * `lua` - The Lua context.
/// * `job_name` - The name of the job this step belongs to.
/// * `step` - The Lua table representing the step.
/// * `runtime_env` - The environment variables to use for the step.
/// 
/// # Errors
/// Returns an error if the step fails to execute.
fn run_step(
    lua: &Lua,
    job_name: &str,
    step: &Table,
    runtime_env: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step-level run function
    if let Ok(func) = step.get::<mlua::Function>("run") {
        let env_table = lua.create_table()?;
        for (k, v) in runtime_env {
            env_table.set(k.as_str(), v.as_str())?;
        }
        func.call::<Table>(env_table)?;
        return Ok(());
    }

    // Step-level run string
    if let Ok(cmd) = step.get::<String>("run") {
        run_command(job_name, &cmd, runtime_env)?;
        return Ok(());
    }

    eprintln!("⚠️ Unsupported step type for job '{}'", job_name);
    Ok(())
}

/// Runs a shell command as part of a job or step, using the provided environment.
/// 
/// # Arguments
/// * `job_name` - The name of the job this command belongs to.
/// * `cmd` - The shell command to execute.
/// * `runtime_env` - The environment variables to use for the command.
/// 
/// # Errors
/// Returns an error if the command fails to execute.
fn run_command(
    job_name: &str,
    cmd: &str,
    runtime_env: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .envs(runtime_env)
            .status()?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .envs(runtime_env)
            .status()?
    };
    if !status.success() {
        error!("❌ Command failed in job '{}'", job_name);
    }
    Ok(())
}