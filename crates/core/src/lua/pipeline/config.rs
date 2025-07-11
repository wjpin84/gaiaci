//! # Pipeline and Step Configuration
//! 
//! This module defines configuration structures for pipelines and steps,
//! including execution options, timeouts, and parallel processing settings.

use serde::{Deserialize, Serialize};

/// Configuration options for pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Maximum execution time in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    
    /// Whether to continue on step failure
    #[serde(default)]
    pub continue_on_error: bool,
    
    /// Working directory for the pipeline
    #[serde(default)]
    pub working_directory: Option<String>,
    
    /// Parallel execution settings
    #[serde(default)]
    pub parallel: ParallelConfig,
    
    /// Global environment variables for all steps
    #[serde(default)]
    pub global_env: std::collections::HashMap<String, String>,
    
    /// Maximum memory usage in bytes
    #[serde(default)]
    pub max_memory_bytes: Option<u64>,
    
    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
}

/// Parallel execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Whether steps can run in parallel
    #[serde(default)]
    pub enabled: bool,
    
    /// Maximum number of concurrent steps
    #[serde(default = "default_max_parallel")]
    pub max_concurrent: usize,
    
    /// Timeout for waiting on dependencies (seconds)
    #[serde(default = "default_dependency_timeout")]
    pub dependency_timeout_seconds: u64,
    
    /// Strategy for handling failed steps in parallel execution
    #[serde(default)]
    pub failure_strategy: FailureStrategy,
}

/// Retry configuration for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default)]
    pub max_attempts: u32,
    
    /// Delay between retries in seconds
    #[serde(default = "default_retry_delay")]
    pub delay_seconds: u64,
    
    /// Whether to use exponential backoff
    #[serde(default)]
    pub exponential_backoff: bool,
    
    /// Maximum delay between retries (for exponential backoff)
    #[serde(default = "default_max_retry_delay")]
    pub max_delay_seconds: u64,
    
    /// Conditions under which to retry (exit codes, error patterns, etc.)
    #[serde(default)]
    pub retry_conditions: Vec<RetryCondition>,
}

/// Conditions under which a step should be retried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// Retry on any failure
    AnyFailure,
    
    /// Retry on specific exit codes
    ExitCodes(Vec<i32>),
    
    /// Retry if error message contains pattern
    ErrorPattern(String),
    
    /// Retry on timeout
    Timeout,
    
    /// Custom Lua condition
    LuaCondition(String),
}

/// Strategy for handling failures in parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureStrategy {
    /// Stop all execution on first failure
    FailFast,
    
    /// Continue execution but mark pipeline as failed
    ContinueOnFailure,
    
    /// Only fail if critical steps fail
    CriticalOnly,
    
    /// Custom strategy defined by Lua function
    Custom(String),
}

// Default value functions
fn default_timeout() -> u64 { 3600 } // 1 hour
fn default_max_parallel() -> usize { 4 }
fn default_dependency_timeout() -> u64 { 300 } // 5 minutes
fn default_retry_delay() -> u64 { 1 } // 1 second
fn default_max_retry_delay() -> u64 { 60 } // 1 minute

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            continue_on_error: false,
            working_directory: None,
            parallel: ParallelConfig::default(),
            global_env: std::collections::HashMap::new(),
            max_memory_bytes: None,
            verbose: false,
        }
    }
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_concurrent: default_max_parallel(),
            dependency_timeout_seconds: default_dependency_timeout(),
            failure_strategy: FailureStrategy::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 0,
            delay_seconds: default_retry_delay(),
            exponential_backoff: false,
            max_delay_seconds: default_max_retry_delay(),
            retry_conditions: vec![RetryCondition::AnyFailure],
        }
    }
}

impl Default for FailureStrategy {
    fn default() -> Self {
        FailureStrategy::FailFast
    }
}

impl PipelineConfig {
    /// Creates a new pipeline configuration with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the timeout for the pipeline
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Enables or disables continuing on errors
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }
    
    /// Sets the working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
    
    /// Configures parallel execution
    pub fn with_parallel(mut self, config: ParallelConfig) -> Self {
        self.parallel = config;
        self
    }
    
    /// Adds a global environment variable
    pub fn with_global_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.global_env.insert(key.into(), value.into());
        self
    }
    
    /// Enables verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl ParallelConfig {
    /// Creates a new parallel configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enables parallel execution with the specified concurrency
    pub fn with_concurrency(mut self, max_concurrent: usize) -> Self {
        self.enabled = true;
        self.max_concurrent = max_concurrent;
        self
    }
    
    /// Sets the dependency timeout
    pub fn with_dependency_timeout(mut self, seconds: u64) -> Self {
        self.dependency_timeout_seconds = seconds;
        self
    }
    
    /// Sets the failure strategy
    pub fn with_failure_strategy(mut self, strategy: FailureStrategy) -> Self {
        self.failure_strategy = strategy;
        self
    }
}

impl RetryConfig {
    /// Creates a new retry configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the maximum number of retry attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }
    
    /// Sets the delay between retries
    pub fn with_delay(mut self, seconds: u64) -> Self {
        self.delay_seconds = seconds;
        self
    }
    
    /// Enables exponential backoff
    pub fn with_exponential_backoff(mut self, enabled: bool) -> Self {
        self.exponential_backoff = enabled;
        self
    }
    
    /// Sets the maximum delay for exponential backoff
    pub fn with_max_delay(mut self, seconds: u64) -> Self {
        self.max_delay_seconds = seconds;
        self
    }
    
    /// Adds a retry condition
    pub fn with_condition(mut self, condition: RetryCondition) -> Self {
        self.retry_conditions.push(condition);
        self
    }
}
