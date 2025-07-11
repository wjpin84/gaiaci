//! # Task Scheduling for Parallel Execution
//!
//! Intelligent scheduling of pipeline tasks based on dependencies and resource constraints.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::lua::pipeline::step::{Pipeline, PipelineStep};
use crate::lua::pipeline::validator::PipelineValidator;

use super::{ParallelError, ParallelResult};
use super::resource::{ResourceAnalyzer, ResourceRequirements};
use super::config::ParallelConfig;

/// A group of tasks that can be executed together
#[derive(Debug, Clone)]
pub struct TaskGroup {
    /// Steps in this group
    pub steps: Vec<String>,
    
    /// Whether steps in this group can run in parallel
    pub can_run_parallel: bool,
    
    /// Combined resource requirements
    pub resource_requirements: ResourceRequirements,
    
    /// Estimated execution duration
    pub estimated_duration: Option<Duration>,
    
    /// Group priority (highest priority step in group)
    pub priority: u8,
}

/// Complete execution plan for a pipeline
#[derive(Debug)]
pub struct ExecutionPlan {
    /// Ordered groups of tasks
    pub task_groups: Vec<TaskGroup>,
    
    /// Total estimated duration
    pub estimated_total_duration: Option<Duration>,
    
    /// Maximum parallelism level
    pub max_parallelism: usize,
    
    /// Critical path through the pipeline
    pub critical_path: Vec<String>,
}

/// Intelligent task scheduler
pub struct ExecutionScheduler {
    config: ParallelConfig,
}

impl ExecutionScheduler {
    /// Create a new scheduler with the given configuration
    pub fn new(config: ParallelConfig) -> ParallelResult<Self> {
        config.validate()
            .map_err(|e| ParallelError::Scheduling(e))?;
        
        Ok(Self { config })
    }
    
    /// Create an optimized execution plan for the pipeline
    pub fn create_execution_plan(&self, pipeline: &Pipeline) -> ParallelResult<ExecutionPlan> {
        // Validate pipeline structure
        pipeline.validate()
            .map_err(|e| ParallelError::Pipeline(e.to_string()))?;
        
        // Get dependency-ordered steps
        let ordered_steps = PipelineValidator::order_steps_by_dependencies(&pipeline.steps)
            .map_err(|e| ParallelError::Pipeline(e.to_string()))?;
        
        // Analyze resource requirements for all steps
        let resource_map = ResourceAnalyzer::analyze_pipeline(pipeline);
        
        // Group steps by dependency levels and resource constraints
        let task_groups = self.create_task_groups(&ordered_steps, pipeline, &resource_map)?;
        
        // Calculate execution plan metrics
        let estimated_total_duration = self.calculate_total_duration(&task_groups);
        let max_parallelism = task_groups.iter()
            .map(|group| if group.can_run_parallel { group.steps.len() } else { 1 })
            .max()
            .unwrap_or(1);
        
        // Find critical path
        let critical_path = self.find_critical_path(&task_groups);
        
        Ok(ExecutionPlan {
            task_groups,
            estimated_total_duration,
            max_parallelism,
            critical_path,
        })
    }
    
    /// Group steps into executable task groups
    fn create_task_groups(
        &self,
        ordered_steps: &[&PipelineStep],
        pipeline: &Pipeline,
        resource_map: &HashMap<String, ResourceRequirements>,
    ) -> ParallelResult<Vec<TaskGroup>> {
        let mut groups = Vec::new();
        let mut processed_steps = HashSet::new();
        let mut current_level = Vec::new();
        
        // Group steps by dependency levels
        for step in ordered_steps {
            let all_deps_processed = step.depends_on.iter()
                .all(|dep| processed_steps.contains(dep));
            
            if all_deps_processed {
                // Can be added to current level
                current_level.push(step.name.clone());
            } else {
                // Start new level
                if !current_level.is_empty() {
                    let group = self.create_task_group(&current_level, pipeline, resource_map)?;
                    groups.push(group);
                    
                    // Mark all steps in this group as processed
                    for step_name in &current_level {
                        processed_steps.insert(step_name.clone());
                    }
                    current_level.clear();
                }
                current_level.push(step.name.clone());
            }
        }
        
        // Handle remaining steps
        if !current_level.is_empty() {
            let group = self.create_task_group(&current_level, pipeline, resource_map)?;
            groups.push(group);
        }
        
        Ok(groups)
    }
    
    /// Create a task group from a set of step names
    fn create_task_group(
        &self,
        step_names: &[String],
        pipeline: &Pipeline,
        resource_map: &HashMap<String, ResourceRequirements>,
    ) -> ParallelResult<TaskGroup> {
        // Get steps and their requirements
        let steps: Vec<_> = step_names.iter()
            .filter_map(|name| pipeline.get_step(name))
            .collect();
        
        let requirements: Vec<_> = step_names.iter()
            .filter_map(|name| resource_map.get(name))
            .cloned()
            .collect();
        
        if steps.is_empty() {
            return Err(ParallelError::Scheduling("Empty task group".to_string()));
        }
        
        // Determine if steps can run in parallel
        let can_run_parallel = step_names.len() > 1 && 
            self.can_steps_run_parallel(&steps, &requirements);
        
        // Calculate combined requirements
        let combined_requirements = ResourceRequirements::combine(&requirements);
        
        // Calculate estimated duration
        let estimated_duration = self.estimate_group_duration(&steps, can_run_parallel);
        
        // Determine group priority
        let priority = requirements.iter()
            .map(|r| r.priority)
            .max()
            .unwrap_or(50);
        
        Ok(TaskGroup {
            steps: step_names.to_vec(),
            can_run_parallel,
            resource_requirements: combined_requirements,
            estimated_duration,
            priority,
        })
    }
    
    /// Check if steps can run in parallel based on configuration and resources
    fn can_steps_run_parallel(
        &self,
        steps: &[&PipelineStep],
        requirements: &[ResourceRequirements],
    ) -> bool {
        // Check for exclusive access requirement
        if requirements.iter().any(|r| r.requires_exclusive_access) {
            return false;
        }
        
        // Check resource constraints
        let max_cpu_cores = num_cpus::get() as f64 * 
            (self.config.resources.max_cpu_percent.unwrap_or(100.0) / 100.0);
        
        let max_memory_mb = self.config.resources.max_memory_mb
            .unwrap_or(u64::MAX);
        
        let max_io_tasks = self.config.resources.max_concurrent_io_tasks
            .unwrap_or(usize::MAX);
        
        ResourceAnalyzer::can_run_parallel(
            steps,
            max_cpu_cores,
            max_memory_mb,
            max_io_tasks,
        )
    }
    
    /// Estimate execution duration for a group
    fn estimate_group_duration(
        &self,
        steps: &[&PipelineStep],
        can_run_parallel: bool,
    ) -> Option<Duration> {
        let durations: Vec<_> = steps.iter()
            .filter_map(|step| step.timeout_seconds.map(Duration::from_secs))
            .collect();
        
        if durations.is_empty() {
            return None;
        }
        
        Some(if can_run_parallel {
            // Parallel execution: limited by longest task
            durations.into_iter().max().unwrap()
        } else {
            // Sequential execution: sum of all tasks
            durations.into_iter().sum()
        })
    }
    
    /// Calculate total estimated duration for the execution plan
    fn calculate_total_duration(&self, groups: &[TaskGroup]) -> Option<Duration> {
        let group_durations: Vec<_> = groups.iter()
            .filter_map(|group| group.estimated_duration)
            .collect();
        
        if group_durations.is_empty() {
            None
        } else {
            Some(group_durations.into_iter().sum())
        }
    }
    
    /// Find the critical path through the execution plan
    fn find_critical_path(&self, groups: &[TaskGroup]) -> Vec<String> {
        // Simple implementation: return steps from groups with highest priority
        // and longest duration
        let mut critical_steps = Vec::new();
        
        for group in groups {
            if let Some(duration) = group.estimated_duration {
                // Add steps from groups that are on the critical path
                if group.priority >= 70 || duration >= Duration::from_secs(60) {
                    critical_steps.extend(group.steps.iter().cloned());
                }
            }
        }
        
        // If no critical steps found, include all steps
        if critical_steps.is_empty() {
            critical_steps = groups.iter()
                .flat_map(|group| group.steps.iter().cloned())
                .collect();
        }
        
        critical_steps
    }
}

impl ExecutionPlan {
    /// Get the total number of steps in the plan
    pub fn total_steps(&self) -> usize {
        self.task_groups.iter()
            .map(|group| group.steps.len())
            .sum()
    }
    
    /// Get steps that can run immediately (no dependencies)
    pub fn get_ready_steps(&self) -> Vec<String> {
        self.task_groups.first()
            .map(|group| group.steps.clone())
            .unwrap_or_default()
    }
    
    /// Check if the plan is suitable for parallel execution
    pub fn benefits_from_parallelism(&self) -> bool {
        self.max_parallelism > 1 && 
        self.task_groups.iter().any(|group| group.can_run_parallel)
    }
    
    /// Get execution summary as a human-readable string
    pub fn summary(&self) -> String {
        let total_steps = self.total_steps();
        let parallel_groups = self.task_groups.iter()
            .filter(|group| group.can_run_parallel)
            .count();
        
        let duration_str = self.estimated_total_duration
            .map(|d| format!("~{:.1}s", d.as_secs_f64()))
            .unwrap_or_else(|| "unknown".to_string());
        
        format!(
            "Execution Plan: {} steps in {} groups ({} parallel), estimated duration: {}, max parallelism: {}",
            total_steps,
            self.task_groups.len(),
            parallel_groups,
            duration_str,
            self.max_parallelism
        )
    }
}

impl TaskGroup {
    /// Check if this group contains the given step
    pub fn contains_step(&self, step_name: &str) -> bool {
        self.steps.iter().any(|name| name == step_name)
    }
    
    /// Get the primary step (first or highest priority)
    pub fn primary_step(&self) -> &str {
        self.steps.first()
            .map(|s| s.as_str())
            .unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;
    use crate::lua::pipeline::parallel::config::ParallelConfig;

    fn create_test_pipeline() -> Pipeline {
        let mut pipeline = Pipeline::new("test");
        pipeline.add_step(PipelineStep::new("step1"));
        pipeline.add_step(PipelineStep::new("step2").with_dependency("step1"));
        pipeline.add_step(PipelineStep::new("step3").with_dependency("step1"));
        pipeline.add_step(PipelineStep::new("step4").with_dependency("step2").with_dependency("step3"));
        pipeline
    }

    #[test]
    fn test_execution_plan_creation() {
        let config = ParallelConfig::minimal();
        let scheduler = ExecutionScheduler::new(config).unwrap();
        let pipeline = create_test_pipeline();
        
        let plan = scheduler.create_execution_plan(&pipeline).unwrap();
        
        assert!(plan.task_groups.len() >= 3); // At least 3 levels of dependencies
        assert_eq!(plan.total_steps(), 4);
        assert!(plan.max_parallelism >= 1);
    }
    
    #[test]
    fn test_parallel_detection() {
        let config = ParallelConfig::auto_configure();
        let scheduler = ExecutionScheduler::new(config).unwrap();
        let pipeline = create_test_pipeline();
        
        let plan = scheduler.create_execution_plan(&pipeline).unwrap();
        
        // Step2 and step3 should be able to run in parallel
        let parallel_group = plan.task_groups.iter()
            .find(|group| group.steps.len() > 1);
        
        if let Some(group) = parallel_group {
            assert!(group.can_run_parallel);
            assert!(group.steps.contains(&"step2".to_string()));
            assert!(group.steps.contains(&"step3".to_string()));
        }
    }
    
    #[test]
    fn test_plan_summary() {
        let config = ParallelConfig::minimal();
        let scheduler = ExecutionScheduler::new(config).unwrap();
        let pipeline = create_test_pipeline();
        
        let plan = scheduler.create_execution_plan(&pipeline).unwrap();
        let summary = plan.summary();
        
        assert!(summary.contains("4 steps"));
        assert!(summary.contains("groups"));
    }
    
    #[test]
    fn test_critical_path() {
        let config = ParallelConfig::auto_configure();
        let scheduler = ExecutionScheduler::new(config).unwrap();
        let mut pipeline = Pipeline::new("test");
        
        // Create a pipeline with a clear critical path
        pipeline.add_step(
            PipelineStep::new("critical")
                .with_timeout(120) // Long running
                .with_critical(true)
        );
        pipeline.add_step(PipelineStep::new("fast").with_timeout(5));
        
        let plan = scheduler.create_execution_plan(&pipeline).unwrap();
        
        assert!(plan.critical_path.contains(&"critical".to_string()));
    }
}
