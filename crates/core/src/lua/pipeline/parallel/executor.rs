//! # Clean Parallel Executor Implementation
//!
//! A maintainable, testable parallel executor that follows clean code principles.

use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

use crate::lua::pipeline::{PipelineError, step::{Pipeline, PipelineStep}};
use crate::lua::pipeline::executor::{PipelineExecutor, StepExecutionContext, StepExecutionResult};

use super::{ParallelError, ParallelResult};
use super::config::ParallelConfig;
use super::scheduler::{ExecutionPlan, ExecutionScheduler, TaskGroup};
use super::worker::WorkerPool;

/// Clean, maintainable parallel executor
pub struct ParallelExecutor {
    config: ParallelConfig,
    scheduler: ExecutionScheduler,
    sequential_executor: PipelineExecutor,
}

/// Execution context for parallel execution
#[derive(Debug)]
pub struct ParallelExecutionContext {
    pub plan: ExecutionPlan,
    pub start_time: Instant,
    pub total_steps: usize,
    pub completed_steps: usize,
}

/// Results from parallel execution
#[derive(Debug)]
pub struct ParallelExecutionResult {
    pub success: bool,
    pub total_duration: std::time::Duration,
    pub steps_executed: usize,
    pub steps_failed: usize,
    pub steps_skipped: usize,
    pub parallel_efficiency: f64, // 0.0 to 1.0
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: ParallelConfig) -> ParallelResult<Self> {
        let scheduler = ExecutionScheduler::new(config.clone())?;
        
        Ok(Self {
            config,
            scheduler,
            sequential_executor: PipelineExecutor::new(),
        })
    }
    
    /// Create with auto-configured settings
    pub fn auto_configure() -> ParallelResult<Self> {
        Self::new(ParallelConfig::auto_configure())
    }
    
    /// Create optimized for CI/CD environments
    pub fn for_ci() -> ParallelResult<Self> {
        Self::new(ParallelConfig::ci_optimized())
    }
    
    /// Execute a pipeline with optimal parallelization
    pub async fn execute_pipeline<F>(
        &mut self,
        pipeline: &Pipeline,
        step_executor: F,
    ) -> ParallelResult<ParallelExecutionResult>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError> 
           + Send + Sync + Clone + 'static,
    {
        let start_time = Instant::now();
        
        // Create execution plan
        let plan = self.scheduler.create_execution_plan(pipeline)?;
        
        let result = if plan.benefits_from_parallelism() {
            self.execute_parallel(&plan, pipeline, step_executor).await?
        } else {
            self.execute_sequential(&plan, pipeline, step_executor)?
        };
        
        let total_duration = start_time.elapsed();
        
        Ok(ParallelExecutionResult {
            success: result.steps_failed == 0,
            total_duration,
            steps_executed: result.steps_executed,
            steps_failed: result.steps_failed,
            steps_skipped: result.steps_skipped,
            parallel_efficiency: self.calculate_efficiency(&plan, total_duration),
        })
    }
    
    /// Execute using parallel workers
    async fn execute_parallel<F>(
        &self,
        plan: &ExecutionPlan,
        pipeline: &Pipeline,
        _step_executor: F,
    ) -> ParallelResult<ExecutionStats>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError> 
           + Send + Sync + Clone + 'static,
    {
        let mut stats = ExecutionStats::default();
        
        // Create worker pool
        let worker_pool_config = super::worker::WorkerPoolConfig {
            max_workers: self.config.worker.count,
            task_timeout: self.config.worker.task_timeout,
            shutdown_timeout: Duration::from_secs(30),
            max_retries: 3,
        };
        let mut worker_pool = WorkerPool::new(worker_pool_config);
        worker_pool.start().await?;
        
        // Create cancellation token
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        
        // Execute task groups sequentially, but tasks within groups in parallel
        for group in &plan.task_groups {
            if *cancel_rx.borrow() {
                break;
            }
            
            let group_stats = if group.can_run_parallel {
                self.execute_group_parallel(&worker_pool, group, pipeline, &cancel_rx).await?
            } else {
                self.execute_group_sequential(&worker_pool, group, pipeline, &cancel_rx).await?
            };
            
            stats.merge(&group_stats);
            
            // Stop on critical failure
            if group_stats.has_critical_failure {
                break;
            }
        }
        
        // Graceful shutdown
        worker_pool.shutdown().await?;
        
        Ok(stats)
    }
    
    /// Fallback to sequential execution
    fn execute_sequential<F>(
        &mut self,
        _plan: &ExecutionPlan,
        pipeline: &Pipeline,
        step_executor: F,
    ) -> ParallelResult<ExecutionStats>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError>,
    {
        // Use the sequential executor
        self.sequential_executor.execute_pipeline(pipeline, step_executor)
            .map_err(|e| ParallelError::Pipeline(e.to_string()))?;
        
        let summary = self.sequential_executor.execution_summary();
        
        Ok(ExecutionStats {
            steps_executed: summary.successful_steps as usize,
            steps_failed: summary.failed_steps as usize,
            steps_skipped: summary.skipped_steps as usize,
            has_critical_failure: summary.failed_steps > 0,
        })
    }
    
    /// Execute a single group in parallel
    async fn execute_group_parallel(
        &self,
        worker_pool: &WorkerPool,
        group: &TaskGroup,
        pipeline: &Pipeline,
        cancel_rx: &watch::Receiver<bool>,
    ) -> ParallelResult<ExecutionStats> {
        let mut stats = ExecutionStats::default();
        
        // Create channels for task results
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();
        
        // Submit all tasks in the group
        for step_name in &group.steps {
            if let Some(step) = pipeline.get_step(step_name) {
                let context = StepExecutionContext {
                    step_name: step_name.clone(),
                    attempt: 0,
                    start_time: Instant::now(),
                    timeout: step.timeout_seconds.map(std::time::Duration::from_secs),
                    env_overrides: std::collections::HashMap::new(),
                };
                
                let task = super::worker::WorkerTask {
                    step: step.clone(),
                    context: context.clone(),
                    priority: if step.critical { 100 } else { 50 },
                    result_sender: result_tx.clone(),
                };
                worker_pool.submit_task(task).await?;
            }
        }
        drop(result_tx); // Close sender
        
        // Collect results
        let mut completed = 0;
        while completed < group.steps.len() && !*cancel_rx.borrow() {
            if let Some(task_result) = result_rx.recv().await {
                match task_result.result {
                    Ok(_) => stats.steps_executed += 1,
                    Err(_) => {
                        stats.steps_failed += 1;
                        if pipeline.get_step(&group.steps[completed])
                            .map(|s| s.critical)
                            .unwrap_or(false) 
                        {
                            stats.has_critical_failure = true;
                        }
                    }
                }
                completed += 1;
            } else {
                break;
            }
        }
        
        Ok(stats)
    }
    
    /// Execute a single group sequentially
    async fn execute_group_sequential(
        &self,
        worker_pool: &WorkerPool,
        group: &TaskGroup,
        pipeline: &Pipeline,
        cancel_rx: &watch::Receiver<bool>,
    ) -> ParallelResult<ExecutionStats> {
        let mut stats = ExecutionStats::default();
        
        for step_name in &group.steps {
            if *cancel_rx.borrow() {
                break;
            }
            
            if let Some(step) = pipeline.get_step(step_name) {
                let context = StepExecutionContext {
                    step_name: step_name.clone(),
                    attempt: 0,
                    start_time: Instant::now(),
                    timeout: step.timeout_seconds.map(std::time::Duration::from_secs),
                    env_overrides: std::collections::HashMap::new(),
                };
                
                let (result_tx, mut result_rx) = mpsc::unbounded_channel();
                let task = super::worker::WorkerTask {
                    step: step.clone(),
                    context: context.clone(),
                    priority: if step.critical { 100 } else { 50 },
                    result_sender: result_tx,
                };
                worker_pool.submit_task(task).await?;
                
                if let Some(task_result) = result_rx.recv().await {
                    match task_result.result {
                        Ok(_) => stats.steps_executed += 1,
                        Err(_) => {
                            stats.steps_failed += 1;
                            if step.critical {
                                stats.has_critical_failure = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(stats)
    }
    
    /// Calculate parallel execution efficiency
    fn calculate_efficiency(&self, plan: &ExecutionPlan, actual_duration: std::time::Duration) -> f64 {
        if let Some(estimated_duration) = plan.estimated_total_duration {
            if actual_duration.as_secs_f64() > 0.0 {
                (estimated_duration.as_secs_f64() / actual_duration.as_secs_f64()).min(1.0)
            } else {
                1.0
            }
        } else {
            0.5 // Unknown efficiency
        }
    }
    
    /// Get current execution configuration
    pub fn config(&self) -> &ParallelConfig {
        &self.config
    }
    
    /// Get execution statistics from sequential executor
    pub fn sequential_stats(&self) -> &PipelineExecutor {
        &self.sequential_executor
    }
}

/// Internal execution statistics
#[derive(Debug, Default, Clone)]
struct ExecutionStats {
    steps_executed: usize,
    steps_failed: usize,
    steps_skipped: usize,
    has_critical_failure: bool,
}

impl ExecutionStats {
    fn merge(&mut self, other: &ExecutionStats) {
        self.steps_executed += other.steps_executed;
        self.steps_failed += other.steps_failed;
        self.steps_skipped += other.steps_skipped;
        self.has_critical_failure |= other.has_critical_failure;
    }
}

impl std::fmt::Display for ParallelExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Parallel Execution Result: {} (executed: {}, failed: {}, skipped: {}, duration: {:.2}s, efficiency: {:.1}%)",
            if self.success { "SUCCESS" } else { "FAILED" },
            self.steps_executed,
            self.steps_failed,
            self.steps_skipped,
            self.total_duration.as_secs_f64(),
            self.parallel_efficiency * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[tokio::test]
    async fn test_parallel_executor_creation() {
        let executor = ParallelExecutor::auto_configure().unwrap();
        assert!(executor.config.worker.count > 0);
    }
    
    #[tokio::test]
    async fn test_simple_pipeline_execution() {
        let mut pipeline = Pipeline::new("test");
        pipeline.add_step(PipelineStep::new("step1"));
        
        let mut executor = ParallelExecutor::auto_configure().unwrap();
        
        let result = executor.execute_pipeline(&pipeline, |_step, _context| {
            Ok(StepExecutionResult::success(
                std::time::Duration::from_millis(100),
                Some("success".to_string())
            ))
        }).await;
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.steps_executed, 1);
    }
    
    #[test]
    fn test_efficiency_calculation() {
        let executor = ParallelExecutor::auto_configure().unwrap();
        let plan = ExecutionPlan {
            task_groups: vec![],
            estimated_total_duration: Some(std::time::Duration::from_secs(10)),
            max_parallelism: 2,
            critical_path: vec![],
        };
        
        let efficiency = executor.calculate_efficiency(&plan, std::time::Duration::from_secs(8));
        assert!(efficiency > 1.0); // Should be capped at 1.0
        
        let efficiency = executor.calculate_efficiency(&plan, std::time::Duration::from_secs(15));
        assert!(efficiency < 1.0);
    }
}
