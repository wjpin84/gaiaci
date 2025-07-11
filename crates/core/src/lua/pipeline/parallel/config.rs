//! # Parallel Execution Configuration
//!
//! Clean, type-safe configuration for parallel pipeline execution.

use std::time::Duration;
use serde::{Deserialize, Serialize};

use super::constants::*;

/// Configuration for parallel pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Worker pool configuration
    pub worker: WorkerConfig,
    
    /// Resource management configuration
    pub resources: ResourceLimits,
    
    /// Execution behavior configuration
    pub execution: ExecutionConfig,
}

/// Worker pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Number of worker threads
    pub count: usize,
    
    /// Maximum time a worker can run a single task
    pub task_timeout: Duration,
    
    /// Interval for health checks
    pub health_check_interval: Duration,
    
    /// How often workers poll for new tasks
    pub poll_interval: Duration,
    
    /// Enable worker isolation (separate processes)
    pub enable_isolation: bool,
}

/// Resource limits for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage (0-100)
    pub max_cpu_percent: Option<f64>,
    
    /// Maximum memory per worker in MB
    pub max_memory_mb: Option<u64>,
    
    /// Maximum disk I/O rate in MB/s
    pub max_disk_io_mb_per_sec: Option<u64>,
    
    /// Maximum concurrent network connections
    pub max_network_connections: Option<u32>,
    
    /// Maximum concurrent I/O intensive tasks
    pub max_concurrent_io_tasks: Option<usize>,
}

/// Execution behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum steps that can run concurrently
    pub max_concurrent_steps: usize,
    
    /// Enable adaptive scheduling based on system load
    pub adaptive_scheduling: bool,
    
    /// Prefer CPU cores over logical cores for CPU-intensive tasks
    pub prefer_physical_cores: bool,
    
    /// Enable task priority scheduling
    pub enable_priority_scheduling: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            worker: WorkerConfig::default(),
            resources: ResourceLimits::default(),
            execution: ExecutionConfig::default(),
        }
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            count: (cpu_count).clamp(MIN_WORKER_COUNT, MAX_WORKER_COUNT),
            task_timeout: DEFAULT_WORKER_TIMEOUT,
            health_check_interval: DEFAULT_HEALTH_CHECK_INTERVAL,
            poll_interval: DEFAULT_QUEUE_POLL_INTERVAL,
            enable_isolation: false, // Disabled by default for simplicity
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: Some(DEFAULT_CPU_LIMIT_PERCENT),
            max_memory_mb: Some(DEFAULT_MEMORY_LIMIT_MB),
            max_disk_io_mb_per_sec: None, // Unlimited by default
            max_network_connections: Some(DEFAULT_MAX_CONNECTIONS),
            max_concurrent_io_tasks: Some(2), // Conservative default
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            max_concurrent_steps: cpu_count * 2, // 2x oversubscription
            adaptive_scheduling: true,
            prefer_physical_cores: true,
            enable_priority_scheduling: true,
        }
    }
}

impl ParallelConfig {
    /// Create a new configuration optimized for the current system
    pub fn auto_configure() -> Self {
        let cpu_count = num_cpus::get();
        let physical_cores = num_cpus::get_physical();
        
        Self {
            worker: WorkerConfig {
                count: physical_cores.clamp(MIN_WORKER_COUNT, MAX_WORKER_COUNT),
                ..WorkerConfig::default()
            },
            execution: ExecutionConfig {
                max_concurrent_steps: if cpu_count > 8 { cpu_count } else { cpu_count * 2 },
                ..ExecutionConfig::default()
            },
            ..Self::default()
        }
    }
    
    /// Create a configuration optimized for CI/CD environments
    pub fn ci_optimized() -> Self {
        Self {
            worker: WorkerConfig {
                task_timeout: Duration::from_secs(600), // 10 minutes for CI
                enable_isolation: true, // Better for CI reliability
                ..WorkerConfig::default()
            },
            resources: ResourceLimits {
                max_cpu_percent: Some(90.0), // More aggressive in CI
                max_memory_mb: Some(2048),   // 2GB per worker
                max_concurrent_io_tasks: Some(4), // More I/O in CI
                ..ResourceLimits::default()
            },
            execution: ExecutionConfig {
                adaptive_scheduling: false, // Predictable behavior in CI
                ..ExecutionConfig::default()
            },
            ..Self::default()
        }
    }
    
    /// Create a minimal configuration for testing
    pub fn minimal() -> Self {
        Self {
            worker: WorkerConfig {
                count: 1,
                task_timeout: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(1),
                poll_interval: Duration::from_millis(10),
                enable_isolation: false,
            },
            resources: ResourceLimits {
                max_cpu_percent: Some(50.0),
                max_memory_mb: Some(256),
                max_network_connections: Some(10),
                max_concurrent_io_tasks: Some(1),
                ..ResourceLimits::default()
            },
            execution: ExecutionConfig {
                max_concurrent_steps: 1,
                adaptive_scheduling: false,
                prefer_physical_cores: false,
                enable_priority_scheduling: false,
            },
        }
    }
    
    /// Validate the configuration and return any issues
    pub fn validate(&self) -> Result<(), String> {
        if self.worker.count == 0 {
            return Err("Worker count must be greater than 0".to_string());
        }
        
        if self.worker.count > MAX_WORKER_COUNT {
            return Err(format!("Worker count {} exceeds maximum {}", 
                self.worker.count, MAX_WORKER_COUNT));
        }
        
        if let Some(cpu_percent) = self.resources.max_cpu_percent {
            if cpu_percent <= 0.0 || cpu_percent > 100.0 {
                return Err("CPU percentage must be between 0 and 100".to_string());
            }
        }
        
        if self.execution.max_concurrent_steps == 0 {
            return Err("Max concurrent steps must be greater than 0".to_string());
        }
        
        Ok(())
    }
}
