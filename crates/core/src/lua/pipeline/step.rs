//! # Pipeline Step Definition
//! 
//! This module defines the core Pipeline and PipelineStep types along with
//! their builder patterns and utility methods.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::lua::pipeline::{PipelineConfig, RetryConfig, PipelineError};

/// Represents a complete CI/CD pipeline loaded from Lua
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline name/identifier
    pub name: String,
    
    /// Optional pipeline description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Environment variables for the pipeline
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// Pipeline steps to execute
    pub steps: Vec<PipelineStep>,
    
    /// Pipeline configuration options
    #[serde(default)]
    pub config: PipelineConfig,
    
    /// Pipeline metadata (tags, labels, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    
    /// Pipeline version/revision
    #[serde(default)]
    pub version: Option<String>,
}

/// Represents a single step in a CI/CD pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Step name/identifier
    pub name: String,
    
    /// Optional step description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Conditions for step execution
    #[serde(default)]
    pub condition: Option<String>,
    
    /// Step-specific environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// Whether this step can run in parallel with others
    #[serde(default)]
    pub parallel: bool,
    
    /// Steps that must complete before this one
    #[serde(default)]
    pub depends_on: Vec<String>,
    
    /// The step action (stored as execution reference)
    #[serde(skip)]
    pub action: Option<String>, // Store as string for execution via runtime
    
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
    
    /// Step timeout (overrides pipeline timeout)
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    
    /// Whether this step is critical (pipeline fails if it fails)
    #[serde(default)]
    pub critical: bool,
    
    /// Step metadata and tags
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    
    /// Working directory for this step (relative to pipeline working directory)
    #[serde(default)]
    pub working_directory: Option<String>,
}

/// Step execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    /// Step has not started yet
    Pending,
    
    /// Step is currently running
    Running,
    
    /// Step completed successfully
    Success,
    
    /// Step failed
    Failed,
    
    /// Step was skipped due to conditions or dependencies
    Skipped,
    
    /// Step was cancelled
    Cancelled,
    
    /// Step is waiting for dependencies
    Waiting,
}

impl Pipeline {
    /// Creates a new pipeline with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            env: HashMap::new(),
            steps: Vec::new(),
            config: PipelineConfig::default(),
            metadata: HashMap::new(),
            version: None,
        }
    }
    
    /// Sets the pipeline description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Sets the pipeline version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
    
    /// Sets the pipeline configuration
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Adds a step to the pipeline
    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
    }
    
    /// Adds a step to the pipeline (builder pattern)
    pub fn with_step(mut self, step: PipelineStep) -> Self {
        self.steps.push(step);
        self
    }
    
    /// Adds an environment variable to the pipeline
    pub fn add_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }
    
    /// Adds an environment variable (builder pattern)
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    
    /// Adds metadata to the pipeline
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// Adds metadata (builder pattern)
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Gets a step by name
    pub fn get_step(&self, name: &str) -> Option<&PipelineStep> {
        self.steps.iter().find(|step| step.name == name)
    }
    
    /// Gets a mutable step by name
    pub fn get_step_mut(&mut self, name: &str) -> Option<&mut PipelineStep> {
        self.steps.iter_mut().find(|step| step.name == name)
    }
    
    /// Gets all step names
    pub fn step_names(&self) -> Vec<&String> {
        self.steps.iter().map(|step| &step.name).collect()
    }
    
    /// Gets steps that have no dependencies
    pub fn root_steps(&self) -> Vec<&PipelineStep> {
        self.steps.iter()
            .filter(|step| step.depends_on.is_empty())
            .collect()
    }
    
    /// Gets steps that depend on the given step
    pub fn dependent_steps(&self, step_name: &str) -> Vec<&PipelineStep> {
        self.steps.iter()
            .filter(|step| step.depends_on.contains(&step_name.to_string()))
            .collect()
    }
    
    /// Validates the pipeline structure
    pub fn validate(&self) -> Result<(), PipelineError> {
        use crate::lua::pipeline::PipelineValidator;
        PipelineValidator::validate(self)
    }
    
    /// Gets the total estimated execution time (if steps have timing metadata)
    pub fn estimated_duration(&self) -> Option<u64> {
        let mut total = 0u64;
        let mut has_estimates = false;
        
        for step in &self.steps {
            if let Some(estimate) = step.metadata.get("estimated_duration_seconds") {
                if let Ok(duration) = estimate.parse::<u64>() {
                    total += duration;
                    has_estimates = true;
                }
            }
        }
        
        if has_estimates { Some(total) } else { None }
    }
    
    /// Counts steps by status (if status metadata is available)
    pub fn step_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for step in &self.steps {
            let status = step.metadata.get("status").unwrap_or(&"pending".to_string()).clone();
            *counts.entry(status).or_insert(0) += 1;
        }
        
        counts
    }
}

impl PipelineStep {
    /// Creates a new pipeline step
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            condition: None,
            env: HashMap::new(),
            parallel: false,
            depends_on: Vec::new(),
            action: None,
            retry: RetryConfig::default(),
            timeout_seconds: None,
            critical: false,
            metadata: HashMap::new(),
            working_directory: None,
        }
    }
    
    /// Sets the step description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Sets the step condition
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
    
    /// Adds a dependency
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.depends_on.push(dep.into());
        self
    }
    
    /// Adds multiple dependencies
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on.extend(deps);
        self
    }
    
    /// Adds an environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    
    /// Sets the action reference
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
    
    /// Sets the retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
    
    /// Marks the step as critical
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
    
    /// Enables parallel execution
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
    
    /// Sets the step timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
    
    /// Sets the working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
    
    /// Adds metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Checks if the step can run (all dependencies are met)
    pub fn can_run(&self, completed_steps: &[String]) -> bool {
        self.depends_on.iter().all(|dep| completed_steps.contains(dep))
    }
    
    /// Gets the effective timeout (step timeout or pipeline default)
    pub fn effective_timeout(&self, pipeline_timeout: u64) -> u64 {
        self.timeout_seconds.unwrap_or(pipeline_timeout)
    }
    
    /// Checks if this step should be retried based on its configuration
    pub fn should_retry(&self, attempt: u32, error: &str, exit_code: Option<i32>) -> bool {
        if attempt >= self.retry.max_attempts {
            return false;
        }
        
        use crate::lua::pipeline::config::RetryCondition;
        
        self.retry.retry_conditions.iter().any(|condition| {
            match condition {
                RetryCondition::AnyFailure => true,
                RetryCondition::ExitCodes(codes) => {
                    exit_code.map_or(false, |code| codes.contains(&code))
                }
                RetryCondition::ErrorPattern(pattern) => {
                    error.contains(pattern)
                }
                RetryCondition::Timeout => {
                    error.contains("timeout") || error.contains("timed out")
                }
                RetryCondition::LuaCondition(_) => {
                    // This would need to be evaluated by the runtime
                    false
                }
            }
        })
    }
    
    /// Calculates the delay before the next retry attempt
    pub fn retry_delay(&self, attempt: u32) -> u64 {
        if self.retry.exponential_backoff {
            let base_delay = self.retry.delay_seconds;
            let delay = base_delay * 2_u64.pow(attempt);
            delay.min(self.retry.max_delay_seconds)
        } else {
            self.retry.delay_seconds
        }
    }
}

impl Default for StepStatus {
    fn default() -> Self {
        StepStatus::Pending
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "pending"),
            StepStatus::Running => write!(f, "running"),
            StepStatus::Success => write!(f, "success"),
            StepStatus::Failed => write!(f, "failed"),
            StepStatus::Skipped => write!(f, "skipped"),
            StepStatus::Cancelled => write!(f, "cancelled"),
            StepStatus::Waiting => write!(f, "waiting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = Pipeline::new("test-pipeline")
            .with_description("A test pipeline")
            .with_version("1.0.0")
            .with_env_var("TEST_VAR", "test_value")
            .with_metadata("team", "core");
        
        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.description, Some("A test pipeline".to_string()));
        assert_eq!(pipeline.version, Some("1.0.0".to_string()));
        assert_eq!(pipeline.env.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(pipeline.metadata.get("team"), Some(&"core".to_string()));
    }

    #[test]
    fn test_step_creation() {
        let step = PipelineStep::new("test-step")
            .with_description("A test step")
            .with_condition("true")
            .with_dependency("setup")
            .with_env_var("STEP_VAR", "value")
            .with_critical(true)
            .with_parallel(true)
            .with_timeout(300);
        
        assert_eq!(step.name, "test-step");
        assert_eq!(step.description, Some("A test step".to_string()));
        assert_eq!(step.condition, Some("true".to_string()));
        assert_eq!(step.depends_on, vec!["setup"]);
        assert_eq!(step.env.get("STEP_VAR"), Some(&"value".to_string()));
        assert!(step.critical);
        assert!(step.parallel);
        assert_eq!(step.timeout_seconds, Some(300));
    }

    #[test]
    fn test_step_can_run() {
        let step = PipelineStep::new("test")
            .with_dependency("dep1")
            .with_dependency("dep2");
        
        assert!(!step.can_run(&[]));
        assert!(!step.can_run(&["dep1".to_string()]));
        assert!(step.can_run(&["dep1".to_string(), "dep2".to_string()]));
        assert!(step.can_run(&["dep1".to_string(), "dep2".to_string(), "extra".to_string()]));
    }

    #[test]
    fn test_retry_delay() {
        let step = PipelineStep::new("test")
            .with_retry(RetryConfig::new()
                .with_delay(2)
                .with_exponential_backoff(true)
                .with_max_delay(60));
        
        assert_eq!(step.retry_delay(0), 2);
        assert_eq!(step.retry_delay(1), 4);
        assert_eq!(step.retry_delay(2), 8);
        assert_eq!(step.retry_delay(10), 60); // Should cap at max_delay
    }

    #[test]
    fn test_pipeline_dependent_steps() {
        let mut pipeline = Pipeline::new("test");
        pipeline.add_step(PipelineStep::new("step1"));
        pipeline.add_step(PipelineStep::new("step2").with_dependency("step1"));
        pipeline.add_step(PipelineStep::new("step3").with_dependency("step1"));
        pipeline.add_step(PipelineStep::new("step4").with_dependency("step2"));
        
        let dependents = pipeline.dependent_steps("step1");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.iter().any(|s| s.name == "step2"));
        assert!(dependents.iter().any(|s| s.name == "step3"));
        
        let step4_deps = pipeline.dependent_steps("step2");
        assert_eq!(step4_deps.len(), 1);
        assert_eq!(step4_deps[0].name, "step4");
    }
}
