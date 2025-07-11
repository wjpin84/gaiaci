//! # Resource Management for Parallel Execution
//!
//! Handles resource analysis, requirements calculation, and constraint enforcement.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::lua::pipeline::step::{Pipeline, PipelineStep};
use super::constants::*;

/// Resource requirements for pipeline execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required (can be fractional for light workloads)
    pub cpu_cores: f64,
    
    /// Memory required in MB
    pub memory_mb: u64,
    
    /// Whether this requires intensive disk I/O
    pub disk_io_intensive: bool,
    
    /// Whether this requires intensive network I/O
    pub network_io_intensive: bool,
    
    /// Whether this step requires exclusive system access
    pub requires_exclusive_access: bool,
    
    /// Priority level (0-100, higher = more important)
    pub priority: u8,
}

/// Resource hints extracted from step environment variables
#[derive(Debug, Clone)]
pub struct StepResourceHints {
    pub cpu_intensive: bool,
    pub memory_intensive: bool,
    pub io_intensive: bool,
    pub network_intensive: bool,
    pub exclusive_access: bool,
    pub custom_priority: Option<u8>,
}

/// Analyzes resource requirements for pipeline steps
pub struct ResourceAnalyzer;

impl ResourceAnalyzer {
    /// Analyze resource requirements for a single step
    pub fn analyze_step(step: &PipelineStep) -> ResourceRequirements {
        let hints = Self::extract_hints(step);
        
        let mut requirements = ResourceRequirements {
            priority: if step.critical { 90 } else { 50 },
            ..Default::default()
        };
        
        // Apply custom priority if specified
        if let Some(priority) = hints.custom_priority {
            requirements.priority = priority;
        }
        
        // Calculate CPU requirements
        requirements.cpu_cores = if hints.cpu_intensive {
            1.0 // Full core for CPU-intensive tasks
        } else {
            0.25 // Light CPU usage for most tasks
        };
        
        // Calculate memory requirements
        requirements.memory_mb = if hints.memory_intensive {
            1024 // 1GB for memory-intensive tasks
        } else {
            256 // 256MB baseline
        };
        
        // Set I/O flags
        requirements.disk_io_intensive = hints.io_intensive;
        requirements.network_io_intensive = hints.network_intensive;
        requirements.requires_exclusive_access = hints.exclusive_access;
        
        requirements
    }
    
    /// Analyze resource requirements for an entire pipeline
    pub fn analyze_pipeline(pipeline: &Pipeline) -> HashMap<String, ResourceRequirements> {
        pipeline.steps.iter()
            .map(|step| (step.name.clone(), Self::analyze_step(step)))
            .collect()
    }
    
    /// Check if steps can run in parallel based on resource constraints
    pub fn can_run_parallel(
        steps: &[&PipelineStep],
        max_cpu_cores: f64,
        max_memory_mb: u64,
        max_io_tasks: usize,
    ) -> bool {
        if steps.iter().any(|step| {
            Self::extract_hints(step).exclusive_access
        }) {
            return false; // Exclusive access prevents parallelism
        }
        
        let requirements: Vec<_> = steps.iter()
            .map(|step| Self::analyze_step(step))
            .collect();
        
        // Check CPU constraints
        let total_cpu: f64 = requirements.iter().map(|r| r.cpu_cores).sum();
        if total_cpu > max_cpu_cores {
            return false;
        }
        
        // Check memory constraints
        let total_memory: u64 = requirements.iter().map(|r| r.memory_mb).sum();
        if total_memory > max_memory_mb {
            return false;
        }
        
        // Check I/O constraints
        let io_tasks = requirements.iter()
            .filter(|r| r.disk_io_intensive || r.network_io_intensive)
            .count();
        if io_tasks > max_io_tasks {
            return false;
        }
        
        true
    }
    
    /// Extract resource hints from step environment variables
    fn extract_hints(step: &PipelineStep) -> StepResourceHints {
        StepResourceHints {
            cpu_intensive: Self::env_var_is_true(&step.env, ENV_CPU_INTENSIVE),
            memory_intensive: Self::env_var_is_true(&step.env, ENV_MEMORY_INTENSIVE),
            io_intensive: Self::env_var_is_true(&step.env, ENV_IO_INTENSIVE),
            network_intensive: Self::env_var_is_true(&step.env, ENV_NETWORK_INTENSIVE),
            exclusive_access: Self::env_var_is_true(&step.env, ENV_EXCLUSIVE_ACCESS),
            custom_priority: Self::parse_priority(&step.env),
        }
    }
    
    /// Check if an environment variable indicates a true value
    fn env_var_is_true(env: &HashMap<String, String>, key: &str) -> bool {
        env.get(key)
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
            .unwrap_or(false)
    }
    
    /// Parse custom priority from environment variables
    fn parse_priority(env: &HashMap<String, String>) -> Option<u8> {
        env.get("GAIA_PRIORITY")
            .and_then(|v| v.parse::<u8>().ok())
            .filter(|&p| p <= 100)
    }
}

impl ResourceRequirements {
    /// Create requirements for a CPU-intensive task
    pub fn cpu_intensive() -> Self {
        Self {
            cpu_cores: 1.0,
            memory_mb: 512,
            priority: 70,
            ..Default::default()
        }
    }
    
    /// Create requirements for a memory-intensive task
    pub fn memory_intensive() -> Self {
        Self {
            cpu_cores: 0.5,
            memory_mb: 2048,
            priority: 60,
            ..Default::default()
        }
    }
    
    /// Create requirements for an I/O-intensive task
    pub fn io_intensive() -> Self {
        Self {
            cpu_cores: 0.25,
            memory_mb: 256,
            disk_io_intensive: true,
            priority: 50,
            ..Default::default()
        }
    }
    
    /// Create requirements for an exclusive task
    pub fn exclusive() -> Self {
        Self {
            cpu_cores: num_cpus::get() as f64,
            memory_mb: 4096,
            requires_exclusive_access: true,
            priority: 100,
            ..Default::default()
        }
    }
    
    /// Combine multiple requirements (for parallel execution)
    pub fn combine(requirements: &[Self]) -> Self {
        Self {
            cpu_cores: requirements.iter().map(|r| r.cpu_cores).sum(),
            memory_mb: requirements.iter().map(|r| r.memory_mb).sum(),
            disk_io_intensive: requirements.iter().any(|r| r.disk_io_intensive),
            network_io_intensive: requirements.iter().any(|r| r.network_io_intensive),
            requires_exclusive_access: requirements.iter().any(|r| r.requires_exclusive_access),
            priority: requirements.iter().map(|r| r.priority).max().unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[test]
    fn test_analyze_cpu_intensive_step() {
        let step = PipelineStep::new("cpu_task")
            .with_env_var(ENV_CPU_INTENSIVE, "true");
        
        let requirements = ResourceAnalyzer::analyze_step(&step);
        
        assert_eq!(requirements.cpu_cores, 1.0);
        assert!(requirements.memory_mb >= 256);
    }
    
    #[test]
    fn test_analyze_memory_intensive_step() {
        let step = PipelineStep::new("memory_task")
            .with_env_var(ENV_MEMORY_INTENSIVE, "true");
        
        let requirements = ResourceAnalyzer::analyze_step(&step);
        
        assert_eq!(requirements.memory_mb, 1024);
    }
    
    #[test]
    fn test_exclusive_access_prevents_parallel() {
        let step1 = PipelineStep::new("exclusive")
            .with_env_var(ENV_EXCLUSIVE_ACCESS, "true");
        let step2 = PipelineStep::new("normal");
        
        let can_parallel = ResourceAnalyzer::can_run_parallel(
            &[&step1, &step2], 
            4.0, 
            2048, 
            2
        );
        
        assert!(!can_parallel);
    }
    
    #[test]
    fn test_resource_constraints() {
        let step1 = PipelineStep::new("task1")
            .with_env_var(ENV_CPU_INTENSIVE, "true");
        let step2 = PipelineStep::new("task2")
            .with_env_var(ENV_CPU_INTENSIVE, "true");
        
        // Should fail with only 1 CPU core available
        let can_parallel = ResourceAnalyzer::can_run_parallel(
            &[&step1, &step2], 
            1.0, 
            2048, 
            2
        );
        
        assert!(!can_parallel);
        
        // Should succeed with 2 CPU cores available
        let can_parallel = ResourceAnalyzer::can_run_parallel(
            &[&step1, &step2], 
            2.0, 
            2048, 
            2
        );
        
        assert!(can_parallel);
    }
    
    #[test]
    fn test_combine_requirements() {
        let req1 = ResourceRequirements::cpu_intensive();
        let req2 = ResourceRequirements::memory_intensive();
        
        let combined = ResourceRequirements::combine(&[req1, req2]);
        
        assert_eq!(combined.cpu_cores, 1.5);
        assert_eq!(combined.memory_mb, 2560);
        assert_eq!(combined.priority, 70); // Max of the two
    }
}
