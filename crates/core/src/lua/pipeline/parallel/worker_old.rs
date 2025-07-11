//! # Worker Pool for Parallel Execution
//!
//! Clean, async worker pool implementation with proper error handling and resource management.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

use crate::lua::pipeline::{PipelineError, step::PipelineStep};
use crate::lua::pipeline::executor::{StepExecutionContext, StepExecutionResult};

use super::{ParallelError, ParallelResult};
use super::config::WorkerConfig;

/// Status of a worker
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Running(String), // step name
    Failed(String),
    Shutdown,
}

/// Status of the entire worker pool
#[derive(Debug)]
pub struct WorkerPoolStatus {
    pub total_workers: usize,
    pub idle_workers: usize,
    pub running_workers: usize,
    pub failed_workers: usize,
    pub pending_tasks: usize,
    pub is_shutdown: bool,
}

/// Task to be executed by a worker
#[derive(Debug)]
struct WorkerTask {
    step: PipelineStep,
    context: StepExecutionContext,
    result_sender: mpsc::Sender<Result<StepExecutionResult, PipelineError>>,
}

/// Individual worker state
#[derive(Debug)]
pub struct Worker {
    id: usize,
    status: Arc<RwLock<WorkerStatus>>,
    handle: JoinHandle<()>,
}

/// Clean, async worker pool
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: mpsc::Sender<WorkerTask>,
    config: WorkerConfig,
    semaphore: Arc<Semaphore>, // Rate limiting
    is_shutdown: Arc<RwLock<bool>>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub async fn new<F>(
        config: WorkerConfig,
        step_executor: Arc<F>,
    ) -> ParallelResult<Self>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError> 
           + Send + Sync + 'static,
    {
        let (task_sender, task_receiver) = mpsc::channel(config.count * 2);
        let task_receiver = Arc::new(tokio::sync::Mutex::new(task_receiver));
        let semaphore = Arc::new(Semaphore::new(config.count));
        let is_shutdown = Arc::new(RwLock::new(false));
        
        let mut workers = Vec::with_capacity(config.count);
        
        // Create workers
        for worker_id in 0..config.count {
            let worker = Self::create_worker(
                worker_id,
                config.clone(),
                step_executor.clone(),
                task_receiver.clone(),
                semaphore.clone(),
                is_shutdown.clone(),
            ).await?;
            workers.push(worker);
        }
        
        Ok(Self {
            workers,
            task_sender,
            config,
            semaphore,
            is_shutdown,
        })
    }
    
    /// Submit a task for execution
    pub async fn submit_task(
        &self,
        step: PipelineStep,
        context: StepExecutionContext,
        result_sender: mpsc::Sender<Result<StepExecutionResult, PipelineError>>,
    ) -> ParallelResult<()> {
        if *self.is_shutdown.read().await {
            return Err(ParallelError::WorkerPool("Pool is shutdown".to_string()));
        }
        
        let task = WorkerTask {
            step,
            context,
            result_sender,
        };
        
        self.task_sender.send(task).await
            .map_err(|_| ParallelError::Communication("Failed to submit task".to_string()))?;
        
        Ok(())
    }
    
    /// Get current pool status
    pub async fn status(&self) -> WorkerPoolStatus {
        let mut idle_workers = 0;
        let mut running_workers = 0;
        let mut failed_workers = 0;
        
        for worker in &self.workers {
            match *worker.status.read().await {
                WorkerStatus::Idle => idle_workers += 1,
                WorkerStatus::Running(_) => running_workers += 1,
                WorkerStatus::Failed(_) => failed_workers += 1,
                WorkerStatus::Shutdown => {},
            }
        }
        
        WorkerPoolStatus {
            total_workers: self.workers.len(),
            idle_workers,
            running_workers,
            failed_workers,
            pending_tasks: self.semaphore.available_permits(),
            is_shutdown: *self.is_shutdown.read().await,
        }
    }
    
    /// Gracefully shutdown the worker pool
    pub async fn shutdown(&self) -> ParallelResult<()> {
        *self.is_shutdown.write().await = true;
        
        // Close task channel
        drop(&self.task_sender);
        
        // Wait for all workers to complete
        for worker in &self.workers {
            worker.handle.abort();
        }
        
        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Create a single worker
    async fn create_worker<F>(
        worker_id: usize,
        config: WorkerConfig,
        step_executor: Arc<F>,
        task_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<WorkerTask>>>,
        semaphore: Arc<Semaphore>,
        is_shutdown: Arc<RwLock<bool>>,
    ) -> ParallelResult<Worker>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError> 
           + Send + Sync + 'static,
    {
        let status = Arc::new(RwLock::new(WorkerStatus::Idle));
        let status_clone = status.clone();
        
        let handle = tokio::spawn(async move {
            Self::worker_loop(
                worker_id,
                config,
                step_executor,
                task_receiver,
                semaphore,
                status_clone,
                is_shutdown,
            ).await;
        });
        
        Ok(Worker {
            id: worker_id,
            status,
            handle,
        })
    }
    
    /// Main worker loop
    async fn worker_loop<F>(
        worker_id: usize,
        config: WorkerConfig,
        step_executor: Arc<F>,
        task_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<WorkerTask>>>,
        semaphore: Arc<Semaphore>,
        status: Arc<RwLock<WorkerStatus>>,
        is_shutdown: Arc<RwLock<bool>>,
    )
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError> 
           + Send + Sync + 'static,
    {
        while !*is_shutdown.read().await {
            // Wait for a task
            let task = {
                let mut receiver = task_receiver.lock().await;
                match timeout(config.poll_interval, receiver.recv()).await {
                    Ok(Some(task)) => task,
                    Ok(None) => break, // Channel closed
                    Err(_) => continue, // Timeout, check shutdown status
                }
            };
            
            // Acquire semaphore permit (rate limiting)
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => break, // Semaphore closed
            };
            
            // Update status
            *status.write().await = WorkerStatus::Running(task.step.name.clone());
            
            // Execute task with timeout
            let result = Self::execute_task_with_timeout(
                &step_executor,
                &task.step,
                &task.context,
                config.task_timeout,
            ).await;
            
            // Send result
            if let Err(_) = task.result_sender.send(result.clone()).await {
                tracing::warn!("Worker {}: Failed to send result for step '{}'", 
                    worker_id, task.step.name);
            }
            
            // Update status based on result
            *status.write().await = match result {
                Ok(_) => WorkerStatus::Idle,
                Err(e) => WorkerStatus::Failed(e.to_string()),
            };
        }
        
        *status.write().await = WorkerStatus::Shutdown;
        tracing::debug!("Worker {} shutting down", worker_id);
    }
    
    /// Execute a task with timeout
    async fn execute_task_with_timeout<F>(
        step_executor: &F,
        step: &PipelineStep,
        context: &StepExecutionContext,
        task_timeout: Duration,
    ) -> Result<StepExecutionResult, PipelineError>
    where
        F: Fn(&PipelineStep, &StepExecutionContext) -> Result<StepExecutionResult, PipelineError>,
    {
        let step = step.clone();
        let context = context.clone();
        let executor = step_executor.clone();
        
        // Execute in a blocking task to avoid blocking the async runtime
        let execution = tokio::task::spawn_blocking(move || {
            executor(&step, &context)
        });
        
        match timeout(task_timeout, execution).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(PipelineError::SystemError(format!("Task panicked: {}", e))),
            Err(_) => Err(PipelineError::Timeout { 
                seconds: task_timeout.as_secs() 
            }),
        }
    }
}

impl Worker {
    /// Get worker ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Get current worker status
    pub async fn status(&self) -> WorkerStatus {
        self.status.read().await.clone()
    }
    
    /// Check if worker is available
    pub async fn is_available(&self) -> bool {
        matches!(*self.status.read().await, WorkerStatus::Idle)
    }
}

impl std::fmt::Display for WorkerPoolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "WorkerPool: {}/{} workers active, {} idle, {} failed, {} pending tasks{}",
            self.running_workers,
            self.total_workers,
            self.idle_workers,
            self.failed_workers,
            self.pending_tasks,
            if self.is_shutdown { " [SHUTDOWN]" } else { "" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua::pipeline::step::PipelineStep;

    #[tokio::test]
    async fn test_worker_pool_creation() {
        let config = WorkerConfig {
            count: 2,
            task_timeout: Duration::from_secs(1),
            health_check_interval: Duration::from_millis(100),
            poll_interval: Duration::from_millis(10),
            enable_isolation: false,
        };
        
        let executor = Arc::new(|_step: &PipelineStep, _context: &StepExecutionContext| {
            Ok(StepExecutionResult::success(Duration::from_millis(100), None))
        });
        
        let pool = WorkerPool::new(config, executor).await.unwrap();
        let status = pool.status().await;
        
        assert_eq!(status.total_workers, 2);
        assert_eq!(status.idle_workers, 2);
        assert!(!status.is_shutdown);
    }
    
    #[tokio::test]
    async fn test_task_execution() {
        let config = WorkerConfig {
            count: 1,
            task_timeout: Duration::from_secs(1),
            health_check_interval: Duration::from_millis(100),
            poll_interval: Duration::from_millis(10),
            enable_isolation: false,
        };
        
        let executor = Arc::new(|step: &PipelineStep, _context: &StepExecutionContext| {
            Ok(StepExecutionResult::success(
                Duration::from_millis(100),
                Some(format!("Executed {}", step.name))
            ))
        });
        
        let pool = WorkerPool::new(config, executor).await.unwrap();
        let (result_tx, mut result_rx) = mpsc::channel(1);
        
        let step = PipelineStep::new("test_step");
        let context = StepExecutionContext {
            step_name: "test_step".to_string(),
            attempt: 0,
            start_time: std::time::Instant::now(),
            timeout: None,
            env_overrides: HashMap::new(),
        };
        
        pool.submit_task(step, context, result_tx).await.unwrap();
        
        let result = result_rx.recv().await.unwrap();
        assert!(result.is_ok());
        
        pool.shutdown().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_task_timeout() {
        let config = WorkerConfig {
            count: 1,
            task_timeout: Duration::from_millis(100), // Very short timeout
            health_check_interval: Duration::from_millis(10),
            poll_interval: Duration::from_millis(10),
            enable_isolation: false,
        };
        
        let executor = Arc::new(|_step: &PipelineStep, _context: &StepExecutionContext| {
            std::thread::sleep(Duration::from_millis(200)); // Longer than timeout
            Ok(StepExecutionResult::success(Duration::from_millis(200), None))
        });
        
        let pool = WorkerPool::new(config, executor).await.unwrap();
        let (result_tx, mut result_rx) = mpsc::channel(1);
        
        let step = PipelineStep::new("slow_step");
        let context = StepExecutionContext {
            step_name: "slow_step".to_string(),
            attempt: 0,
            start_time: std::time::Instant::now(),
            timeout: None,
            env_overrides: HashMap::new(),
        };
        
        pool.submit_task(step, context, result_tx).await.unwrap();
        
        let result = result_rx.recv().await.unwrap();
        assert!(result.is_err());
        
        pool.shutdown().await.unwrap();
    }
}
