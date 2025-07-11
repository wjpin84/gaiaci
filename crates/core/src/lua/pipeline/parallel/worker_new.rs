//! # Worker Pool for Parallel Execution
//!
//! Clean, async worker pool implementation with proper error handling and resource management.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

use crate::lua::pipeline::step::PipelineStep;
use crate::lua::pipeline::executor::{StepExecutionContext, StepExecutionResult};

use super::{ParallelError, ParallelResult};

/// Status of a worker
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Running(String), // step name
    Failed(String),  // error message
    Finished,
}

/// A task for worker execution
#[derive(Debug)]
pub struct WorkerTask {
    pub step: PipelineStep,
    pub context: StepExecutionContext,
    pub priority: u32,
    pub result_sender: mpsc::UnboundedSender<TaskResult>,
}

/// Result from task execution
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub step_name: String,
    pub result: ParallelResult<StepExecutionResult>,
    pub worker_id: usize,
    pub execution_time: Duration,
}

/// Status of the worker pool
#[derive(Debug, Clone)]
pub struct WorkerPoolStatus {
    pub total_workers: usize,
    pub idle_workers: usize,
    pub running_workers: usize,
    pub failed_workers: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
}

/// Individual worker in the pool
pub struct Worker {
    pub id: usize,
    pub status: Arc<RwLock<WorkerStatus>>,
    pub handle: JoinHandle<()>,
}

/// Configuration for worker pool
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub max_workers: usize,
    pub task_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub max_retries: u32,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get().max(2),
            task_timeout: Duration::from_secs(300),
            shutdown_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

/// Worker pool for parallel task execution
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: mpsc::UnboundedSender<WorkerTask>,
    shutdown_sender: mpsc::UnboundedSender<()>,
    config: WorkerPoolConfig,
    semaphore: Arc<Semaphore>,
    completed_tasks: Arc<RwLock<usize>>,
}

impl WorkerPool {
    /// Creates a new worker pool
    pub fn new(config: WorkerPoolConfig) -> Self {
        let (task_sender, _task_receiver) = mpsc::unbounded_channel();
        let (shutdown_sender, _shutdown_receiver) = mpsc::unbounded_channel();
        
        Self {
            workers: Vec::new(),
            task_sender,
            shutdown_sender,
            semaphore: Arc::new(Semaphore::new(config.max_workers)),
            config,
            completed_tasks: Arc::new(RwLock::new(0)),
        }
    }

    /// Starts the worker pool
    pub async fn start(&mut self) -> ParallelResult<()> {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        let (shutdown_sender, shutdown_receiver) = mpsc::unbounded_channel();
        
        self.task_sender = task_sender;
        self.shutdown_sender = shutdown_sender;
        
        let task_receiver = Arc::new(tokio::sync::Mutex::new(task_receiver));
        
        for worker_id in 0..self.config.max_workers {
            let status = Arc::new(RwLock::new(WorkerStatus::Idle));
            let worker_task_receiver = task_receiver.clone();
            let worker_status = status.clone();
            let worker_config = self.config.clone();
            let worker_semaphore = self.semaphore.clone();
            let worker_completed = self.completed_tasks.clone();
            let mut worker_shutdown = shutdown_receiver.resubscribe();
            
            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    worker_config,
                    worker_task_receiver,
                    worker_status,
                    worker_semaphore,
                    worker_completed,
                    &mut worker_shutdown,
                ).await;
            });
            
            self.workers.push(Worker {
                id: worker_id,
                status,
                handle,
            });
        }
        
        Ok(())
    }

    /// Worker loop for processing tasks
    async fn worker_loop(
        worker_id: usize,
        config: WorkerPoolConfig,
        task_receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<WorkerTask>>>,
        status: Arc<RwLock<WorkerStatus>>,
        semaphore: Arc<Semaphore>,
        completed_tasks: Arc<RwLock<usize>>,
        shutdown_receiver: &mut mpsc::UnboundedReceiver<()>,
    ) {
        loop {
            // Check for shutdown signal
            if shutdown_receiver.try_recv().is_ok() {
                *status.write().await = WorkerStatus::Finished;
                break;
            }
            
            // Try to get a task
            let task = {
                let mut receiver = task_receiver.lock().await;
                receiver.try_recv()
            };
            
            match task {
                Ok(task) => {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Update status
                    *status.write().await = WorkerStatus::Running(task.step.name.clone());
                    
                    // Execute task with timeout
                    let start_time = tokio::time::Instant::now();
                    let result = timeout(
                        config.task_timeout,
                        Self::execute_step(&task.step, &task.context)
                    ).await;
                    
                    let execution_time = start_time.elapsed();
                    
                    let task_result = match result {
                        Ok(step_result) => TaskResult {
                            step_name: task.step.name.clone(),
                            result: step_result.map_err(|e| ParallelError::ExecutionError(e.to_string())),
                            worker_id,
                            execution_time,
                        },
                        Err(_) => TaskResult {
                            step_name: task.step.name.clone(),
                            result: Err(ParallelError::TimeoutError(task.step.name.clone())),
                            worker_id,
                            execution_time,
                        },
                    };
                    
                    // Send result
                    if let Err(_) = task.result_sender.send(task_result) {
                        eprintln!("Failed to send task result for worker {}", worker_id);
                    }
                    
                    // Update completed tasks count
                    *completed_tasks.write().await += 1;
                    
                    // Update status back to idle
                    *status.write().await = WorkerStatus::Idle;
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No tasks available, sleep briefly
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    // Channel closed, shutdown worker
                    *status.write().await = WorkerStatus::Finished;
                    break;
                }
            }
        }
    }

    /// Execute a single step (simplified version)
    async fn execute_step(
        step: &PipelineStep,
        context: &StepExecutionContext,
    ) -> Result<StepExecutionResult, String> {
        // Simplified execution - just return success for now
        // In real implementation, this would execute the actual step
        Ok(StepExecutionResult {
            step_name: context.step_name.clone(),
            success: true,
            exit_code: Some(0),
            stdout: Some("Step executed successfully".to_string()),
            stderr: None,
            execution_time: Duration::from_millis(100),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Submits a task to the worker pool
    pub async fn submit_task(&self, task: WorkerTask) -> ParallelResult<()> {
        self.task_sender.send(task)
            .map_err(|_| ParallelError::ChannelError("Task submission failed".to_string()))?;
        Ok(())
    }

    /// Gets the current status of the worker pool
    pub async fn status(&self) -> WorkerPoolStatus {
        let mut idle_workers = 0;
        let mut running_workers = 0;
        let mut failed_workers = 0;
        
        for worker in &self.workers {
            match *worker.status.read().await {
                WorkerStatus::Idle => idle_workers += 1,
                WorkerStatus::Running(_) => running_workers += 1,
                WorkerStatus::Failed(_) => failed_workers += 1,
                WorkerStatus::Finished => {}
            }
        }
        
        WorkerPoolStatus {
            total_workers: self.workers.len(),
            idle_workers,
            running_workers,
            failed_workers,
            queued_tasks: 0, // Would need actual queue length
            completed_tasks: *self.completed_tasks.read().await,
        }
    }

    /// Gracefully shuts down the worker pool
    pub async fn shutdown(self) -> ParallelResult<()> {
        // Send shutdown signal to all workers
        for _ in 0..self.workers.len() {
            let _ = self.shutdown_sender.send(());
        }
        
        // Wait for all workers to finish
        for worker in self.workers {
            let _ = timeout(self.config.shutdown_timeout, worker.handle).await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[tokio::test]
    async fn test_worker_pool_creation() {
        let config = WorkerPoolConfig::default();
        let pool = WorkerPool::new(config);
        
        assert_eq!(pool.workers.len(), 0); // Workers not started yet
    }

    #[tokio::test]
    async fn test_worker_pool_config_default() {
        let config = WorkerPoolConfig::default();
        
        assert!(config.max_workers >= 2);
        assert!(config.task_timeout > Duration::from_secs(0));
        assert!(config.shutdown_timeout > Duration::from_secs(0));
        assert!(config.max_retries > 0);
    }

    #[tokio::test]
    async fn test_worker_pool_start() {
        let config = WorkerPoolConfig::default();
        let mut pool = WorkerPool::new(config);
        
        let result = pool.start().await;
        assert!(result.is_ok());
        assert_eq!(pool.workers.len(), pool.config.max_workers);
        
        // Clean shutdown
        let _ = pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_task_result_clone() {
        let result = TaskResult {
            step_name: "test".to_string(),
            result: Ok(StepExecutionResult {
                step_name: "test".to_string(),
                success: true,
                exit_code: Some(0),
                stdout: Some("success".to_string()),
                stderr: None,
                execution_time: Duration::from_millis(100),
                metadata: std::collections::HashMap::new(),
            }),
            worker_id: 0,
            execution_time: Duration::from_millis(100),
        };
        
        let _cloned = result.clone();
        // Test passes if this compiles
    }

    #[tokio::test]
    async fn test_worker_status() {
        let status = WorkerStatus::Idle;
        let _cloned = status.clone();
        
        assert_eq!(status, WorkerStatus::Idle);
    }
}
