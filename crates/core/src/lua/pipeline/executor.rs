//! # Pipeline Executor
//! 
//! This module provides sequential pipeline execution with comprehensive
//! step tracking, retry logic, and error handling.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use crate::lua::pipeline::{PipelineError};
use crate::lua::pipeline::step::{Pipeline, PipelineStep, StepStatus};
use crate::lua::pipeline::validator::PipelineValidator;

/// Execution statistics for pipelines
#[derive(Debug, Default, Clone)]
pub struct ExecutionStats {
    pub steps_executed: u64,
    pub failed_steps: u64,
    pub skipped_steps: u64,
    pub total_execution_time_ms: u64,
    pub retried_steps: u64,
    pub cancelled_steps: u64,
}

/// Context for step execution
#[derive(Debug, Clone)]
pub struct StepExecutionContext {
    pub step_name: String,
    pub attempt: u32,
    pub start_time: Instant,
    pub timeout: Option<Duration>,
    pub env_overrides: HashMap<String, String>,
}

/// Result of step execution
#[derive(Debug, Clone)]
pub struct StepExecutionResult {
    pub status: StepStatus,
    pub duration: Duration,
    pub attempt: u32,
    pub output: Option<String>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub should_retry: bool,
}

/// Pipeline executor for running pipeline steps sequentially
pub struct PipelineExecutor {
    stats: ExecutionStats,
    step_results: HashMap<String, StepExecutionResult>,
    execution_order: Vec<String>,
    cancelled: bool,
}

/// Execution plan for a pipeline
#[derive(Debug)]
pub struct ExecutionPlan {
    pub ordered_steps: Vec<String>,
    pub estimated_duration: Option<Duration>,
    pub parallel_groups: Vec<Vec<String>>,
    pub critical_path: Vec<String>,
}

impl PipelineExecutor {
    /// Creates a new pipeline executor
    pub fn new() -> Self {
        Self {
            stats: ExecutionStats::default(),
            step_results: HashMap::new(),
            execution_order: Vec::new(),
            cancelled: false,
        }
    }
    
    /// Gets the current execution statistics
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }
    
    /// Gets the results of executed steps
    pub fn step_results(&self) -> &HashMap<String, StepExecutionResult> {
        &self.step_results
    }
    
    /// Gets the execution order of steps
    pub fn execution_order(&self) -> &[String] {
        &self.execution_order
    }
    
    /// Resets execution statistics and state
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
        self.step_results.clear();
        self.execution_order.clear();
        self.cancelled = false;
    }
    
    /// Creates an execution plan for the pipeline
    pub fn create_execution_plan(pipeline: &Pipeline) -> Result<ExecutionPlan, PipelineError> {
        // Validate the pipeline first
        pipeline.validate()?;
        
        // Order steps by dependencies
        let ordered_steps_refs = PipelineValidator::order_steps_by_dependencies(&pipeline.steps)?;
        let ordered_steps: Vec<String> = ordered_steps_refs.iter()
            .map(|step| step.name.clone())
            .collect();
        
        // Calculate estimated duration
        let estimated_duration = pipeline.estimated_duration()
            .map(|secs| Duration::from_secs(secs));
        
        // For now, we'll create simple sequential execution
        // TODO: Implement parallel grouping logic
        let parallel_groups = vec![ordered_steps.clone()];
        
        // TODO: Implement critical path analysis
        let critical_path = ordered_steps.clone();
        
        Ok(ExecutionPlan {
            ordered_steps,
            estimated_duration,
            parallel_groups,
            critical_path,
        })
    }
    
    /// Executes a pipeline sequentially
    pub fn execute_pipeline<F>(
        &mut self, 
        pipeline: &Pipeline, 
        step_executor: F
    ) -> Result<(), PipelineError>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError>,
    {
        let start_time = Instant::now();
        
        // Create execution plan
        let plan = Self::create_execution_plan(pipeline)?;
        
        // Execute steps in order
        for step_name in &plan.ordered_steps {
            if self.cancelled {
                break;
            }
            
            let step = pipeline.get_step(step_name)
                .ok_or_else(|| PipelineError::ParseError(format!("Step '{}' not found", step_name)))?;
            
            let result = self.execute_step_with_retry(step, pipeline, &step_executor)?;
            
            // Handle step result
            match result.status {
                StepStatus::Success => {
                    self.stats.steps_executed += 1;
                }
                StepStatus::Failed => {
                    self.stats.failed_steps += 1;
                    
                    if step.critical && !pipeline.config.continue_on_error {
                        return Err(PipelineError::StepError {
                            step: step.name.clone(),
                            reason: result.error.unwrap_or_else(|| "Step failed".to_string()),
                        });
                    }
                }
                StepStatus::Skipped => {
                    self.stats.skipped_steps += 1;
                }
                StepStatus::Cancelled => {
                    self.stats.cancelled_steps += 1;
                    break;
                }
                _ => {}
            }
            
            self.step_results.insert(step_name.clone(), result);
            self.execution_order.push(step_name.clone());
        }
        
        self.stats.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(())
    }
    
    /// Executes a single step with retry logic
    fn execute_step_with_retry<F>(
        &mut self,
        step: &PipelineStep,
        pipeline: &Pipeline,
        step_executor: &F,
    ) -> Result<StepExecutionResult, PipelineError>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError>,
    {
        let mut attempt = 0;
        let max_attempts = step.retry.max_attempts.max(1); // At least one attempt
        
        loop {
            let context = StepExecutionContext {
                step_name: step.name.clone(),
                attempt,
                start_time: Instant::now(),
                timeout: step.timeout_seconds.map(Duration::from_secs),
                env_overrides: self.build_env_overrides(step, pipeline),
            };
            
            let result = step_executor(step, &context)?;
            
            // Check if we should retry
            if result.status == StepStatus::Failed && 
               attempt < max_attempts - 1 && 
               result.should_retry {
                
                attempt += 1;
                self.stats.retried_steps += 1;
                
                // Wait before retry
                let delay = step.retry_delay(attempt);
                if delay > 0 {
                    std::thread::sleep(Duration::from_secs(delay));
                }
                
                continue;
            }
            
            return Ok(result);
        }
    }
    
    /// Builds environment variable overrides for a step
    fn build_env_overrides(&self, step: &PipelineStep, pipeline: &Pipeline) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // Start with pipeline global env
        env.extend(pipeline.config.global_env.clone());
        
        // Add pipeline env
        env.extend(pipeline.env.clone());
        
        // Add step-specific env (highest priority)
        env.extend(step.env.clone());
        
        env
    }
    
    /// Cancels pipeline execution
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    
    /// Checks if execution was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    
    /// Records a step execution manually (for external executors)
    pub fn record_step_execution(&mut self, step_name: String, result: StepExecutionResult) {
        match result.status {
            StepStatus::Success => self.stats.steps_executed += 1,
            StepStatus::Failed => self.stats.failed_steps += 1,
            StepStatus::Skipped => self.stats.skipped_steps += 1,
            StepStatus::Cancelled => self.stats.cancelled_steps += 1,
            _ => {}
        }
        
        if result.attempt > 0 {
            self.stats.retried_steps += 1;
        }
        
        self.stats.total_execution_time_ms += result.duration.as_millis() as u64;
        self.step_results.insert(step_name.clone(), result);
        self.execution_order.push(step_name);
    }
    
    /// Records a step execution with simple parameters
    pub fn record_step_execution_simple(&mut self, success: bool, duration_ms: u64) {
        self.stats.steps_executed += 1;
        self.stats.total_execution_time_ms += duration_ms;
        
        if !success {
            self.stats.failed_steps += 1;
        }
    }
    
    /// Records a skipped step
    pub fn record_step_skipped(&mut self) {
        self.stats.skipped_steps += 1;
    }
    
    /// Records a retry attempt
    pub fn record_retry(&mut self) {
        self.stats.retried_steps += 1;
    }
    
    /// Gets detailed execution summary
    pub fn execution_summary(&self) -> ExecutionSummary {
        let total_steps = self.stats.steps_executed + self.stats.failed_steps + 
                         self.stats.skipped_steps + self.stats.cancelled_steps;
        
        let success_rate = if total_steps > 0 {
            (self.stats.steps_executed as f64 / total_steps as f64) * 100.0
        } else {
            0.0
        };
        
        ExecutionSummary {
            total_steps,
            successful_steps: self.stats.steps_executed,
            failed_steps: self.stats.failed_steps,
            skipped_steps: self.stats.skipped_steps,
            cancelled_steps: self.stats.cancelled_steps,
            retried_steps: self.stats.retried_steps,
            total_duration: Duration::from_millis(self.stats.total_execution_time_ms),
            success_rate,
            execution_order: self.execution_order.clone(),
        }
    }
}

/// Detailed execution summary
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_steps: u64,
    pub successful_steps: u64,
    pub failed_steps: u64,
    pub skipped_steps: u64,
    pub cancelled_steps: u64,
    pub retried_steps: u64,
    pub total_duration: Duration,
    pub success_rate: f64,
    pub execution_order: Vec<String>,
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl StepExecutionResult {
    /// Creates a successful step result
    pub fn success(duration: Duration, output: Option<String>) -> Self {
        Self {
            status: StepStatus::Success,
            duration,
            attempt: 0,
            output,
            error: None,
            exit_code: Some(0),
            should_retry: false,
        }
    }
    
    /// Creates a failed step result
    pub fn failure(
        duration: Duration, 
        error: String, 
        exit_code: Option<i32>,
        should_retry: bool
    ) -> Self {
        Self {
            status: StepStatus::Failed,
            duration,
            attempt: 0,
            output: None,
            error: Some(error),
            exit_code,
            should_retry,
        }
    }
    
    /// Creates a skipped step result
    pub fn skipped(reason: String) -> Self {
        Self {
            status: StepStatus::Skipped,
            duration: Duration::ZERO,
            attempt: 0,
            output: None,
            error: Some(reason),
            exit_code: None,
            should_retry: false,
        }
    }
    
    /// Creates a cancelled step result
    pub fn cancelled(duration: Duration) -> Self {
        Self {
            status: StepStatus::Cancelled,
            duration,
            attempt: 0,
            output: None,
            error: Some("Step was cancelled".to_string()),
            exit_code: None,
            should_retry: false,
        }
    }
}

impl std::fmt::Display for ExecutionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Pipeline Execution Summary:\n\
             Total Steps: {}\n\
             Successful: {}\n\
             Failed: {}\n\
             Skipped: {}\n\
             Cancelled: {}\n\
             Retried: {}\n\
             Duration: {:.2}s\n\
             Success Rate: {:.1}%",
            self.total_steps,
            self.successful_steps,
            self.failed_steps,
            self.skipped_steps,
            self.cancelled_steps,
            self.retried_steps,
            self.total_duration.as_secs_f64(),
            self.success_rate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[test]
    fn test_execution_stats() {
        let mut executor = PipelineExecutor::new();
        
        executor.record_step_execution_simple(true, 100);
        executor.record_step_execution_simple(false, 200);
        executor.record_step_skipped();
        executor.record_retry();
        
        let stats = executor.stats();
        assert_eq!(stats.steps_executed, 2);
        assert_eq!(stats.failed_steps, 1);
        assert_eq!(stats.skipped_steps, 1);
        assert_eq!(stats.retried_steps, 1);
        assert_eq!(stats.total_execution_time_ms, 300);
    }

    #[test]
    fn test_step_execution_result() {
        let success = StepExecutionResult::success(
            Duration::from_millis(100), 
            Some("Success output".to_string())
        );
        assert_eq!(success.status, StepStatus::Success);
        assert_eq!(success.duration, Duration::from_millis(100));
        assert!(!success.should_retry);
        
        let failure = StepExecutionResult::failure(
            Duration::from_millis(200),
            "Error message".to_string(),
            Some(1),
            true
        );
        assert_eq!(failure.status, StepStatus::Failed);
        assert!(failure.should_retry);
        assert_eq!(failure.exit_code, Some(1));
    }

    #[test]
    fn test_execution_plan_creation() {
        let mut pipeline = Pipeline::new("test");
        pipeline.add_step(PipelineStep::new("step1"));
        pipeline.add_step(PipelineStep::new("step2").with_dependency("step1"));
        
        let plan = PipelineExecutor::create_execution_plan(&pipeline).unwrap();
        assert_eq!(plan.ordered_steps.len(), 2);
        assert_eq!(plan.ordered_steps[0], "step1");
        assert_eq!(plan.ordered_steps[1], "step2");
    }

    #[test]
    fn test_env_overrides() {
        let mut pipeline = Pipeline::new("test")
            .with_env_var("PIPELINE_VAR", "pipeline_value");
        pipeline.config.global_env.insert("GLOBAL_VAR".to_string(), "global_value".to_string());
        
        let step = PipelineStep::new("test_step")
            .with_env_var("STEP_VAR", "step_value")
            .with_env_var("PIPELINE_VAR", "overridden_value"); // Should override pipeline var
        
        let executor = PipelineExecutor::new();
        let env = executor.build_env_overrides(&step, &pipeline);
        
        assert_eq!(env.get("GLOBAL_VAR"), Some(&"global_value".to_string()));
        assert_eq!(env.get("PIPELINE_VAR"), Some(&"overridden_value".to_string())); // Step overrides pipeline
        assert_eq!(env.get("STEP_VAR"), Some(&"step_value".to_string()));
    }

    #[test]
    fn test_execution_summary() {
        let mut executor = PipelineExecutor::new();
        executor.record_step_execution_simple(true, 100);
        executor.record_step_execution_simple(true, 150);
        executor.record_step_execution_simple(false, 200);
        
        let summary = executor.execution_summary();
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.successful_steps, 2);
        assert_eq!(summary.failed_steps, 1);
        assert_eq!(summary.success_rate, 200.0 / 3.0);
        assert_eq!(summary.total_duration, Duration::from_millis(450));
    }
}
