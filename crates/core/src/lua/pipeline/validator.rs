//! # Pipeline Validator
//! 
//! This module provides comprehensive validation for pipelines, including
//! dependency cycle detection, topological sorting, and structural validation.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::lua::pipeline::{PipelineError};
use crate::lua::pipeline::step::{Pipeline, PipelineStep};

/// Pipeline validator for checking pipeline integrity
pub struct PipelineValidator;

/// Dependency graph for analyzing step relationships
#[derive(Debug)]
pub struct DependencyGraph<'a> {
    steps: HashMap<&'a str, &'a PipelineStep>,
    graph: HashMap<&'a str, Vec<&'a str>>,
    reverse_graph: HashMap<&'a str, Vec<&'a str>>,
    in_degree: HashMap<&'a str, usize>,
}

impl PipelineValidator {
    /// Validates a pipeline for correctness
    pub fn validate(pipeline: &Pipeline) -> Result<(), PipelineError> {
        // Basic structural validation
        Self::validate_structure(pipeline)?;
        
        // Dependency validation
        Self::validate_dependencies(pipeline)?;
        
        // Check for dependency cycles
        if Self::has_dependency_cycles(&pipeline.steps) {
            return Err(PipelineError::DependencyCycle);
        }
        
        // Validate parallel execution configuration
        Self::validate_parallel_config(pipeline)?;
        
        // Validate step configurations
        Self::validate_step_configs(pipeline)?;
        
        Ok(())
    }
    
    /// Validates basic pipeline structure
    pub fn validate_structure(pipeline: &Pipeline) -> Result<(), PipelineError> {
        // Check for empty pipeline name
        if pipeline.name.trim().is_empty() {
            return Err(PipelineError::ParseError("Pipeline name cannot be empty".to_string()));
        }
        
        // Check for at least one step
        if pipeline.steps.is_empty() {
            return Err(PipelineError::ParseError("Pipeline must have at least one step".to_string()));
        }
        
        // Check for unique step names
        let mut step_names = HashSet::new();
        for step in &pipeline.steps {
            if step.name.trim().is_empty() {
                return Err(PipelineError::ParseError("Step name cannot be empty".to_string()));
            }
            
            if !step_names.insert(&step.name) {
                return Err(PipelineError::ParseError(
                    format!("Duplicate step name: '{}'", step.name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validates step dependencies
    pub fn validate_dependencies(pipeline: &Pipeline) -> Result<(), PipelineError> {
        let step_names: HashSet<_> = pipeline.steps.iter()
            .map(|s| &s.name)
            .collect();
        
        for step in &pipeline.steps {
            // Check that all dependencies exist
            for dep in &step.depends_on {
                if !step_names.contains(dep) {
                    return Err(PipelineError::DependencyError(
                        format!("Step '{}' depends on non-existent step '{}'", step.name, dep)
                    ));
                }
            }
            
            // Check for self-dependency
            if step.depends_on.contains(&step.name) {
                return Err(PipelineError::DependencyError(
                    format!("Step '{}' cannot depend on itself", step.name)
                ));
            }
            
            // Check for duplicate dependencies
            let mut seen_deps = HashSet::new();
            for dep in &step.depends_on {
                if !seen_deps.insert(dep) {
                    return Err(PipelineError::DependencyError(
                        format!("Step '{}' has duplicate dependency '{}'", step.name, dep)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validates parallel execution configuration
    pub fn validate_parallel_config(pipeline: &Pipeline) -> Result<(), PipelineError> {
        if pipeline.config.parallel.enabled {
            if pipeline.config.parallel.max_concurrent == 0 {
                return Err(PipelineError::ConfigError(
                    "max_concurrent must be greater than 0 when parallel execution is enabled".to_string()
                ));
            }
            
            // Check that there are steps that can actually run in parallel
            let parallel_steps = pipeline.steps.iter()
                .filter(|step| step.parallel)
                .count();
            
            if parallel_steps == 0 {
                return Err(PipelineError::ConfigError(
                    "No steps marked as parallel, but parallel execution is enabled".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validates individual step configurations
    pub fn validate_step_configs(pipeline: &Pipeline) -> Result<(), PipelineError> {
        for step in &pipeline.steps {
            // Validate retry configuration
            if step.retry.max_attempts > 0 {
                if step.retry.delay_seconds == 0 {
                    return Err(PipelineError::ConfigError(
                        format!("Step '{}' has retry enabled but delay_seconds is 0", step.name)
                    ));
                }
                
                if step.retry.exponential_backoff && step.retry.max_delay_seconds < step.retry.delay_seconds {
                    return Err(PipelineError::ConfigError(
                        format!("Step '{}' has max_delay_seconds less than delay_seconds", step.name)
                    ));
                }
            }
            
            // Validate timeout
            if let Some(timeout) = step.timeout_seconds {
                if timeout == 0 {
                    return Err(PipelineError::ConfigError(
                        format!("Step '{}' timeout cannot be 0", step.name)
                    ));
                }
            }
            
            // Validate parallel steps don't have certain incompatible configurations
            if step.parallel {
                if !step.depends_on.is_empty() {
                    // This is actually allowed - parallel steps can have dependencies
                    // They just can't run until dependencies are met
                }
            }
        }
        
        Ok(())
    }
    
    /// Checks for dependency cycles using topological sort approach
    pub fn has_dependency_cycles(steps: &[PipelineStep]) -> bool {
        let graph = DependencyGraph::new(steps);
        !graph.is_acyclic()
    }
    
    /// Orders steps by dependencies using topological sort
    pub fn order_steps_by_dependencies(steps: &[PipelineStep]) -> Result<Vec<&PipelineStep>, PipelineError> {
        let graph = DependencyGraph::new(steps);
        
        if !graph.is_acyclic() {
            return Err(PipelineError::DependencyCycle);
        }
        
        graph.topological_sort()
    }
    
    /// Finds all steps that can run immediately (no unfulfilled dependencies)
    pub fn find_ready_steps<'a>(
        steps: &'a [PipelineStep], 
        completed_steps: &[String]
    ) -> Vec<&'a PipelineStep> {
        steps.iter()
            .filter(|step| {
                step.depends_on.iter().all(|dep| completed_steps.contains(dep))
            })
            .collect()
    }
    
    /// Finds all steps that would become ready if the given step completes
    pub fn find_next_ready_steps<'a>(
        steps: &'a [PipelineStep],
        completed_steps: &[String],
        newly_completed: &str,
    ) -> Vec<&'a PipelineStep> {
        let mut new_completed = completed_steps.to_vec();
        new_completed.push(newly_completed.to_string());
        
        steps.iter()
            .filter(|step| {
                // Step is not already completed
                !completed_steps.iter().any(|comp| comp == &step.name) &&
                step.name != newly_completed &&
                // All dependencies are now met
                step.depends_on.iter().all(|dep| new_completed.contains(dep))
            })
            .collect()
    }
    
    /// Analyzes the critical path through the pipeline
    pub fn analyze_critical_path(pipeline: &Pipeline) -> Result<CriticalPathAnalysis, PipelineError> {
        let ordered_steps = Self::order_steps_by_dependencies(&pipeline.steps)?;
        
        let mut step_earliest_start = HashMap::new();
        let mut step_latest_start = HashMap::new();
        let mut step_durations = HashMap::new();
        
        // Calculate estimated durations (use metadata if available, otherwise default)
        for step in &pipeline.steps {
            let duration = step.metadata.get("estimated_duration_seconds")
                .and_then(|d| d.parse::<u64>().ok())
                .unwrap_or(60); // Default 1 minute
            step_durations.insert(step.name.clone(), duration);
        }
        
        // Forward pass: calculate earliest start times
        for step in &ordered_steps {
            let earliest_start = step.depends_on.iter()
                .map(|dep| {
                    let dep_start = step_earliest_start.get(dep).unwrap_or(&0);
                    let dep_duration = step_durations.get(dep).unwrap_or(&60);
                    dep_start + dep_duration
                })
                .max()
                .unwrap_or(0);
            step_earliest_start.insert(step.name.clone(), earliest_start);
        }
        
        // Calculate total project duration
        let total_duration = ordered_steps.iter()
            .map(|step| {
                let start = step_earliest_start.get(&step.name).unwrap_or(&0);
                let duration = step_durations.get(&step.name).unwrap_or(&60);
                start + duration
            })
            .max()
            .unwrap_or(0);
        
        // Backward pass: calculate latest start times
        for step in ordered_steps.iter().rev() {
            let latest_start = if Self::find_dependent_steps(&pipeline.steps, &step.name).is_empty() {
                // This is a terminal step
                total_duration - step_durations.get(&step.name).unwrap_or(&60)
            } else {
                Self::find_dependent_steps(&pipeline.steps, &step.name).iter()
                    .map(|dep_step| {
                        let dep_latest = step_latest_start.get(&dep_step.name).unwrap_or(&total_duration);
                        *dep_latest
                    })
                    .min()
                    .unwrap_or(total_duration) - step_durations.get(&step.name).unwrap_or(&60)
            };
            step_latest_start.insert(step.name.clone(), latest_start);
        }
        
        // Find critical path (steps with zero slack)
        let mut critical_steps = Vec::new();
        for step in &pipeline.steps {
            let earliest = step_earliest_start.get(&step.name).unwrap_or(&0);
            let latest = step_latest_start.get(&step.name).unwrap_or(&0);
            if earliest == latest {
                critical_steps.push(step.name.clone());
            }
        }
        
        let step_slack: HashMap<String, u64> = step_earliest_start.iter()
            .map(|(name, earliest)| {
                let latest = step_latest_start.get(name).unwrap_or(earliest);
                (name.clone(), latest - earliest)
            })
            .collect();
        
        Ok(CriticalPathAnalysis {
            total_duration,
            critical_steps,
            step_earliest_start,
            step_latest_start,
            step_slack,
        })
    }
    
    /// Finds steps that depend on the given step
    fn find_dependent_steps<'a>(steps: &'a [PipelineStep], step_name: &str) -> Vec<&'a PipelineStep> {
        steps.iter()
            .filter(|step| step.depends_on.contains(&step_name.to_string()))
            .collect()
    }
}

impl<'a> DependencyGraph<'a> {
    /// Creates a new dependency graph from pipeline steps
    pub fn new(steps: &'a [PipelineStep]) -> Self {
        let mut graph = HashMap::new();
        let mut reverse_graph = HashMap::new();
        let mut in_degree = HashMap::new();
        let steps_map: HashMap<&str, &PipelineStep> = steps.iter()
            .map(|step| (step.name.as_str(), step))
            .collect();
        
        // Initialize graph structures
        for step in steps {
            graph.entry(step.name.as_str()).or_insert_with(Vec::new);
            reverse_graph.entry(step.name.as_str()).or_insert_with(Vec::new);
            in_degree.entry(step.name.as_str()).or_insert(0);
        }
        
        // Build edges
        for step in steps {
            for dep in &step.depends_on {
                if let Some(dep_step) = steps_map.get(dep.as_str()) {
                    graph.entry(dep_step.name.as_str()).or_default().push(step.name.as_str());
                    reverse_graph.entry(step.name.as_str()).or_default().push(dep_step.name.as_str());
                    *in_degree.entry(step.name.as_str()).or_insert(0) += 1;
                }
            }
        }
        
        Self {
            steps: steps_map,
            graph,
            reverse_graph,
            in_degree,
        }
    }
    
    /// Checks if the graph is acyclic using Kahn's algorithm
    pub fn is_acyclic(&self) -> bool {
        let mut in_degree = self.in_degree.clone();
        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();
        
        let mut processed = 0;
        
        while let Some(current) = queue.pop_front() {
            processed += 1;
            
            if let Some(neighbors) = self.graph.get(current) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        
        processed == self.steps.len()
    }
    
    /// Performs topological sort
    pub fn topological_sort(&self) -> Result<Vec<&'a PipelineStep>, PipelineError> {
        let mut in_degree = self.in_degree.clone();
        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();
        
        let mut ordered = Vec::new();
        
        while let Some(current) = queue.pop_front() {
            if let Some(&step) = self.steps.get(current) {
                ordered.push(step);
            }
            
            if let Some(neighbors) = self.graph.get(current) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        
        if ordered.len() != self.steps.len() {
            Err(PipelineError::DependencyCycle)
        } else {
            Ok(ordered)
        }
    }
}

/// Results of critical path analysis
#[derive(Debug)]
pub struct CriticalPathAnalysis {
    /// Total duration of the pipeline
    pub total_duration: u64,
    
    /// Steps on the critical path
    pub critical_steps: Vec<String>,
    
    /// Earliest start time for each step
    pub step_earliest_start: HashMap<String, u64>,
    
    /// Latest start time for each step
    pub step_latest_start: HashMap<String, u64>,
    
    /// Slack time for each step (latest - earliest)
    pub step_slack: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[test]
    fn test_dependency_cycle_detection() {
        let steps = vec![
            PipelineStep::new("step1").with_dependency("step2"),
            PipelineStep::new("step2").with_dependency("step3"),
            PipelineStep::new("step3").with_dependency("step1"), // Creates cycle
        ];
        
        assert!(PipelineValidator::has_dependency_cycles(&steps));
    }

    #[test]
    fn test_no_dependency_cycle() {
        let steps = vec![
            PipelineStep::new("step1"),
            PipelineStep::new("step2").with_dependency("step1"),
            PipelineStep::new("step3").with_dependency("step2"),
        ];
        
        assert!(!PipelineValidator::has_dependency_cycles(&steps));
    }

    #[test]
    fn test_step_ordering() {
        let steps = vec![
            PipelineStep::new("step3").with_dependency("step1").with_dependency("step2"),
            PipelineStep::new("step1"),
            PipelineStep::new("step2").with_dependency("step1"),
        ];
        
        let ordered = PipelineValidator::order_steps_by_dependencies(&steps).unwrap();
        let names: Vec<&str> = ordered.iter().map(|s| s.name.as_str()).collect();
        
        // step1 should come first, step2 second, step3 third
        assert_eq!(names[0], "step1");
        assert_eq!(names[1], "step2");
        assert_eq!(names[2], "step3");
    }

    #[test]
    fn test_find_ready_steps() {
        let steps = vec![
            PipelineStep::new("step1"),
            PipelineStep::new("step2").with_dependency("step1"),
            PipelineStep::new("step3").with_dependency("step1"),
            PipelineStep::new("step4").with_dependency("step2").with_dependency("step3"),
        ];
        
        // Initially, only step1 should be ready
        let ready = PipelineValidator::find_ready_steps(&steps, &[]);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].name, "step1");
        
        // After step1 completes, step2 and step3 should be ready
        let ready = PipelineValidator::find_ready_steps(&steps, &["step1".to_string()]);
        assert_eq!(ready.len(), 2);
        let ready_names: Vec<&str> = ready.iter().map(|s| s.name.as_str()).collect();
        assert!(ready_names.contains(&"step2"));
        assert!(ready_names.contains(&"step3"));
        
        // After step1, step2, and step3 complete, step4 should be ready
        let ready = PipelineValidator::find_ready_steps(&steps, &[
            "step1".to_string(), 
            "step2".to_string(), 
            "step3".to_string()
        ]);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].name, "step4");
    }

    #[test]
    fn test_pipeline_validation() {
        // Valid pipeline
        let mut valid_pipeline = Pipeline::new("test");
        valid_pipeline.add_step(PipelineStep::new("step1"));
        valid_pipeline.add_step(PipelineStep::new("step2").with_dependency("step1"));
        assert!(PipelineValidator::validate(&valid_pipeline).is_ok());
        
        // Invalid: duplicate step names
        let mut invalid_pipeline = Pipeline::new("test");
        invalid_pipeline.add_step(PipelineStep::new("step1"));
        invalid_pipeline.add_step(PipelineStep::new("step1"));
        assert!(PipelineValidator::validate(&invalid_pipeline).is_err());
        
        // Invalid: non-existent dependency
        let mut invalid_dep_pipeline = Pipeline::new("test");
        invalid_dep_pipeline.add_step(PipelineStep::new("step1").with_dependency("nonexistent"));
        assert!(PipelineValidator::validate(&invalid_dep_pipeline).is_err());
        
        // Invalid: self-dependency
        let mut self_dep_pipeline = Pipeline::new("test");
        self_dep_pipeline.add_step(PipelineStep::new("step1").with_dependency("step1"));
        assert!(PipelineValidator::validate(&self_dep_pipeline).is_err());
    }

    #[test]
    fn test_find_next_ready_steps() {
        let steps = vec![
            PipelineStep::new("step1"),
            PipelineStep::new("step2").with_dependency("step1"),
            PipelineStep::new("step3").with_dependency("step1"),
            PipelineStep::new("step4").with_dependency("step2").with_dependency("step3"),
        ];
        
        // When step1 completes, step2 and step3 should become ready
        let next_ready = PipelineValidator::find_next_ready_steps(&steps, &[], "step1");
        assert_eq!(next_ready.len(), 2);
        let ready_names: Vec<&str> = next_ready.iter().map(|s| s.name.as_str()).collect();
        assert!(ready_names.contains(&"step2"));
        assert!(ready_names.contains(&"step3"));
        
        // When step2 completes (with step1 already done), no new steps become ready
        let next_ready = PipelineValidator::find_next_ready_steps(
            &steps, 
            &["step1".to_string()], 
            "step2"
        );
        assert_eq!(next_ready.len(), 0);
        
        // When step3 completes (with step1 and step2 already done), step4 becomes ready
        let next_ready = PipelineValidator::find_next_ready_steps(
            &steps, 
            &["step1".to_string(), "step2".to_string()], 
            "step3"
        );
        assert_eq!(next_ready.len(), 1);
        assert_eq!(next_ready[0].name, "step4");
    }
}
