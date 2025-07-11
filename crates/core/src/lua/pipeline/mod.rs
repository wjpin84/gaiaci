//! # Pipeline Module
//! 
//! This module provides comprehensive pipeline management for GaiaCI, including
//! parsing, validation, execution, and configuration management.
//! 
//! ## Architecture
//! 
//! The pipeline module is organized into focused sub-modules:
//! 
//! - **`config`** - Pipeline and step configuration types
//! - **`step`** - Pipeline step definition and utilities
//! - **`parser`** - Lua table to Pipeline parsing logic
//! - **`validator`** - Pipeline validation and dependency resolution
//! - **`executor`** - Pipeline and step execution engine
//! - **`parallel`** - Parallel execution coordination
//! 
//! ## Usage Example
//! 
//! ```rust
//! use gaiaci_core::lua::pipeline::{Pipeline, PipelineStep, PipelineParser};
//! 
//! // Create a pipeline programmatically
//! let mut pipeline = Pipeline::new("my-pipeline");
//! pipeline.add_step(PipelineStep::new("build"));
//! 
//! // Or parse from Lua (via runtime)
//! // let pipeline = runtime.load_pipeline(script)?;
//! 
//! // Validate and execute
//! pipeline.validate()?;
//! // executor.execute(&pipeline)?;
//! ```

pub mod config;
pub mod step;
pub mod parser;
pub mod validator;
pub mod executor;
pub mod parallel;

// Re-export main types for convenience
pub use config::{PipelineConfig, ParallelConfig, RetryConfig};
pub use step::{PipelineStep, Pipeline};
pub use parser::PipelineParser;
pub use validator::PipelineValidator;
pub use executor::{PipelineExecutor, ExecutionStats};
pub use parallel::ParallelExecutor;

/// Errors that can occur during pipeline operations
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Pipeline parsing error: {0}")]
    ParseError(String),
    
    #[error("Step execution failed: {step} - {reason}")]
    StepError { step: String, reason: String },
    
    #[error("Dependency cycle detected in pipeline steps")]
    DependencyCycle,
    
    #[error("Pipeline timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("Step dependency error: {0}")]
    DependencyError(String),
    
    #[error("Lua execution error: {0}")]
    LuaError(#[from] mlua::Error),
    
    #[error("Parallel execution error: {0}")]
    ParallelError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}
