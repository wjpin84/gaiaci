//! # Lua Runtime for GaiaCI
//! 
//! This module provides a Lua runtime environment for executing CI/CD pipeline scripts.
//! It focuses on Lua VM management, environment setup, and coordination with the pipeline
//! execution engine.
//! 
//! ## Responsibilities
//! 
//! - **Lua VM Management** - Initialize and configure the Lua virtual machine
//! - **Standard Library Integration** - Register gaia.* modules with the Lua environment
//! - **Script Execution** - Execute Lua scripts and evaluate expressions
//! - **Environment Setup** - Manage environment variables and runtime configuration
//! - **Error Handling** - Bridge between Lua errors and Rust error types
//! 
//! ## Usage Example
//! 
//! ```rust
//! # use gaiaci_core::lua::runtime::LuaRuntime;
//! # use gaiaci_core::lua::pipeline::Pipeline;
//! 
//! // Create runtime
//! let mut runtime = LuaRuntime::new()?;
//! 
//! // Load a pipeline script
//! let pipeline_script = r#"
//!     pipeline = {
//!         name = "Build and Test",
//!         steps = {
//!             {
//!                 name = "setup",
//!                 action = function()
//!                     gaia.log.info("Setting up environment...")
//!                     gaia.shell.run({cmd = "npm install"})
//!                 end
//!             }
//!         }
//!     }
//! "#;
//! 
//! // Parse and execute
//! let pipeline = runtime.load_pipeline(pipeline_script)?;
//! runtime.execute_pipeline(&pipeline)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use mlua::{Lua, Result as LuaResult, Table};
use std::path::Path;
use std::fs;
use crate::lua::lib::register_gaia_lib;
use crate::lua::pipeline::{Pipeline, PipelineStep, PipelineParser, PipelineExecutor, PipelineError};

/// Runtime execution context for pipelines
pub struct LuaRuntime {
    /// The Lua VM instance
    lua: Lua,
    
    /// Currently loaded pipeline
    current_pipeline: Option<Pipeline>,
    
    /// Pipeline executor for step execution
    executor: PipelineExecutor,
    
    /// Runtime statistics
    stats: RuntimeStats,
}

/// Runtime execution statistics
#[derive(Debug, Default)]
pub struct RuntimeStats {
    pub scripts_loaded: u64,
    pub steps_executed: u64,
    pub total_execution_time_ms: u64,
    pub failed_steps: u64,
    pub retried_steps: u64,
}

/// Errors that can occur during runtime operations
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Lua execution error: {0}")]
    LuaError(#[from] mlua::Error),
    
    #[error("Pipeline error: {0}")]
    PipelineError(#[from] PipelineError),
    
    #[error("Runtime timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for the Lua runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Memory limit for Lua VM in bytes
    pub memory_limit: Option<usize>,
    
    /// CPU time limit for script execution
    pub cpu_limit: Option<std::time::Duration>,
    
    /// Whether to enable debug information
    pub debug_enabled: bool,
    
    /// Custom module search paths
    pub module_paths: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: Some(128 * 1024 * 1024), // 128MB
            cpu_limit: Some(std::time::Duration::from_secs(300)), // 5 minutes
            debug_enabled: false,
            module_paths: vec![],
        }
    }
}

impl LuaRuntime {
    /// Creates a new Lua runtime with the GaiaCI standard library registered
    pub fn new() -> Result<Self, RuntimeError> {
        let lua = Lua::new();
        
        // Register the GaiaCI standard library
        register_gaia_lib(&lua)?;
        
        // Add runtime-specific functions
        Self::register_runtime_functions(&lua)?;
        
        Ok(Self {
            lua,
            current_pipeline: None,
            executor: PipelineExecutor::new(),
            stats: RuntimeStats::default(),
        })
    }
    
    /// Creates a new runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let mut runtime = Self::new()?;
        runtime.configure(config)?;
        Ok(runtime)
    }
    
    /// Loads a pipeline from a Lua script string
    pub fn load_pipeline(&mut self, script: &str) -> Result<Pipeline, RuntimeError> {
        self.stats.scripts_loaded += 1;
        
        // Execute the Lua script
        self.lua.load(script).exec()?;
        
        // Extract the pipeline table
        let globals = self.lua.globals();
        let pipeline_table: Table = globals.get("pipeline")
            .map_err(|_| PipelineError::ParseError("No 'pipeline' table found in script".to_string()))?;
        
        // Parse the pipeline structure using PipelineParser
        let pipeline = PipelineParser::parse_pipeline_table(pipeline_table)?;
        
        // Validate the pipeline
        pipeline.validate()?;
        
        self.current_pipeline = Some(pipeline.clone());
        Ok(pipeline)
    }
    
    /// Loads a pipeline from a file
    pub fn load_pipeline_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Pipeline, RuntimeError> {
        let script = fs::read_to_string(path)?;
        self.load_pipeline(&script)
    }
    
    /// Executes a loaded pipeline
    pub fn execute_pipeline(&mut self, pipeline: &Pipeline) -> Result<(), RuntimeError> {
        let start_time = std::time::Instant::now();
        
        // Set up pipeline environment
        self.setup_pipeline_environment(pipeline)?;
        
        // Execute steps in dependency order
        for step in &pipeline.steps {
            self.execute_step(step, pipeline)?;
        }
        
        self.stats.total_execution_time_ms += start_time.elapsed().as_millis() as u64;
        Ok(())
    }
    
    /// Executes a single pipeline step
    pub fn execute_step(&mut self, step: &PipelineStep, pipeline: &Pipeline) -> Result<(), RuntimeError> {
        let step_start = std::time::Instant::now();
        self.stats.steps_executed += 1;
        
        // Check step condition if specified
        if let Some(condition) = &step.condition {
            if !self.evaluate_condition(condition)? {
                self.executor.record_step_skipped();
                return Ok(()); // Skip step
            }
        }
        
        // Set up step environment
        self.setup_step_environment(step, pipeline)?;
        
        // Execute the step action if it exists
        if let Some(action_ref) = &step.action {
            let script = format!(
                r#"
                -- Step: {}
                local step_name = "{}"
                gaia.log.info("Executing step: " .. step_name)
                
                -- Execute the action function directly
                {}
                
                gaia.log.info("Step '" .. step_name .. "' completed successfully")
                "#,
                step.name, step.name, action_ref
            );
            
            match self.lua.load(&script).exec() {
                Ok(_) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    self.executor.record_step_execution_simple(true, duration);
                    Ok(())
                }
                Err(e) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    self.stats.failed_steps += 1;
                    self.executor.record_step_execution_simple(false, duration);
                    Err(PipelineError::StepError {
                        step: step.name.clone(),
                        reason: e.to_string(),
                    }.into())
                }
            }
        } else {
            // Log that the step has no action
            let script = format!(r#"gaia.log.warn("Step '{}' has no action defined")"#, step.name);
            self.lua.load(&script).exec()?;
            let duration = step_start.elapsed().as_millis() as u64;
            self.executor.record_step_execution_simple(true, duration);
            Ok(())
        }
    }
    
    /// Evaluates a Lua condition expression
    pub fn evaluate_condition(&self, condition: &str) -> Result<bool, RuntimeError> {
        let result: bool = self.lua.load(condition).eval()?;
        Ok(result)
    }
    
    /// Gets the current runtime statistics
    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }
    
    /// Gets the pipeline executor statistics
    pub fn executor_stats(&self) -> &crate::lua::pipeline::ExecutionStats {
        self.executor.stats()
    }
    
    /// Resets runtime statistics
    pub fn reset_stats(&mut self) {
        self.stats = RuntimeStats::default();
        self.executor.reset_stats();
    }
    
    // Private helper methods
    
    fn register_runtime_functions(lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();
        
        // Add runtime utility functions
        let runtime_table = lua.create_table()?;
        
        // Function to get current step context
        runtime_table.set("current_step", lua.create_function(|_, ()| {
            // This would return current step information
            Ok("current_step_placeholder")
        })?)?;
        
        // Function to skip current step
        runtime_table.set("skip_step", lua.create_function(|_, reason: Option<String>| {
            let msg = reason.unwrap_or_else(|| "Step skipped".to_string());
            // This would set a flag to skip the current step
            Ok(msg)
        })?)?;
        
        globals.set("runtime", runtime_table)?;
        Ok(())
    }
    
    fn configure(&mut self, _config: RuntimeConfig) -> Result<(), RuntimeError> {
        // Apply runtime configuration
        // TODO: Implement configuration application
        Ok(())
    }
    
    fn setup_pipeline_environment(&self, pipeline: &Pipeline) -> Result<(), RuntimeError> {
        // Set pipeline-level environment variables using std::env
        for (key, value) in &pipeline.env {
            std::env::set_var(key, value);
        }
        
        // Set working directory if specified
        if let Some(wd) = &pipeline.config.working_directory {
            let script = format!(r#"gaia.shell.run({{cmd = "cd {}"}})"#, wd);
            self.lua.load(&script).exec()?;
        }
        
        Ok(())
    }
    
    fn setup_step_environment(&self, step: &PipelineStep, _pipeline: &Pipeline) -> Result<(), RuntimeError> {
        // Set step-level environment variables using std::env
        for (key, value) in &step.env {
            std::env::set_var(key, value);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = LuaRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_simple_pipeline_loading() {
        let mut runtime = LuaRuntime::new().unwrap();
        
        let script = r#"
            pipeline = {
                name = "Test Pipeline",
                description = "A simple test pipeline",
                steps = {
                    {
                        name = "hello",
                        description = "Say hello",
                        action = function()
                            gaia.log.info("Hello from pipeline!")
                        end
                    }
                }
            }
        "#;
        
        let result = runtime.load_pipeline(script);
        assert!(result.is_ok());
        
        let pipeline = result.unwrap();
        assert_eq!(pipeline.name, "Test Pipeline");
        assert_eq!(pipeline.steps.len(), 1);
        assert_eq!(pipeline.steps[0].name, "hello");
    }

    #[test]
    fn test_pipeline_with_environment() {
        let mut runtime = LuaRuntime::new().unwrap();
        
        let script = r#"
            pipeline = {
                name = "Env Test",
                env = {
                    NODE_ENV = "test",
                    DEBUG = "true"
                },
                steps = {
                    {
                        name = "check_env",
                        action = function()
                            local node_env = gaia.env.get("NODE_ENV")
                            gaia.log.info("NODE_ENV: " .. (node_env or "not set"))
                        end
                    }
                }
            }
        "#;
        
        let result = runtime.load_pipeline(script);
        assert!(result.is_ok());
        
        let pipeline = result.unwrap();
        assert_eq!(pipeline.env.len(), 2);
        assert_eq!(pipeline.env.get("NODE_ENV"), Some(&"test".to_string()));
    }

    #[test]
    fn test_gaia_stdlib_available() {
        let runtime = LuaRuntime::new().unwrap();
        
        // Test that gaia modules are available
        let result = runtime.lua.load(r#"
            return type(gaia) == "table" and
                   type(gaia.log) == "table" and
                   type(gaia.shell) == "table" and
                   type(gaia.fs) == "table" and
                   type(gaia.env) == "table" and
                   type(gaia.assert) == "table"
        "#).eval::<bool>();
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_runtime_functions_available() {
        let runtime = LuaRuntime::new().unwrap();
        
        // Test that runtime functions are available
        let result = runtime.lua.load(r#"
            return type(runtime) == "table" and
                   type(runtime.current_step) == "function" and
                   type(runtime.skip_step) == "function"
        "#).eval::<bool>();
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
