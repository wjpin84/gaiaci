//! # Parallel Pipeline Execution
//!
//! This module provides a clean, maintainable parallel execution system for pipeline steps.
//! It follows clean code principles with focused responsibilities and clear interfaces.

pub mod config;
pub mod executor;
pub mod resource;
pub mod scheduler;
pub mod worker;

// Re-export main types for convenience
pub use config::{ParallelConfig, ResourceLimits, WorkerConfig};
pub use executor::ParallelExecutor;
pub use resource::{ResourceAnalyzer, ResourceRequirements, StepResourceHints};
pub use scheduler::{ExecutionPlan, ExecutionScheduler, TaskGroup};
pub use worker::{Worker, WorkerPool, WorkerStatus};

/// Result type for parallel execution operations
pub type ParallelResult<T> = Result<T, ParallelError>;

/// Errors that can occur during parallel execution
#[derive(Debug, thiserror::Error)]
pub enum ParallelError {
    #[error("Worker pool error: {0}")]
    WorkerPool(String),

    #[error("Resource constraint violation: {0}")]
    ResourceConstraint(String),

    #[error("Task scheduling error: {0}")]
    Scheduling(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Task timeout: {0}")]
    TimeoutError(String),

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Parallel execution was cancelled")]
    Cancelled,

    #[error("Pipeline error: {0}")]
    Pipeline(String),
}

/// Constants for parallel execution
pub mod constants {
    use std::time::Duration;

    pub const DEFAULT_WORKER_COUNT: usize = 4;
    pub const MIN_WORKER_COUNT: usize = 1;
    pub const MAX_WORKER_COUNT: usize = 32;
    
    pub const DEFAULT_WORKER_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(10);
    pub const DEFAULT_QUEUE_POLL_INTERVAL: Duration = Duration::from_millis(100);
    
    pub const DEFAULT_CPU_LIMIT_PERCENT: f64 = 80.0;
    pub const DEFAULT_MEMORY_LIMIT_MB: u64 = 1024; // 1GB per worker
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 100;
    
    // Environment variable keys for resource hints
    pub const ENV_CPU_INTENSIVE: &str = "GAIA_CPU_INTENSIVE";
    pub const ENV_MEMORY_INTENSIVE: &str = "GAIA_MEMORY_INTENSIVE";
    pub const ENV_IO_INTENSIVE: &str = "GAIA_IO_INTENSIVE";
    pub const ENV_NETWORK_INTENSIVE: &str = "GAIA_NETWORK_INTENSIVE";
    pub const ENV_EXCLUSIVE_ACCESS: &str = "GAIA_EXCLUSIVE_ACCESS";
}
