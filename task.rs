use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

use crate::errors::TasklineError;
use crate::cron_parser::CronSchedule;
use crate::Result;

/// Represents the status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is idle waiting for its next execution
    Idle,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task is paused (will not be executed by scheduler)
    Paused,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Idle => write!(f, "idle"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Paused => write!(f, "paused"),
        }
    }
}

/// TaskConfig defines configuration options for task execution
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Maximum execution time before the task is timed out
    pub timeout: Option<Duration>,
    /// Maximum number of retries on failure
    pub max_retries: u32,
    /// Delay between retries
    pub retry_delay: Duration,
    /// Whether to continue the scheduler if this task fails permanently
    pub fail_scheduler_on_error: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        TaskConfig {
            timeout: Some(Duration::from_secs(60)), // 1 minute timeout by default
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            fail_scheduler_on_error: false,
        }
    }
}

/// Statistics about task execution
#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    /// Number of times the task has been executed
    pub executions: u64,
    /// Number of successful executions
    pub successes: u64,
    /// Number of failed executions
    pub failures: u64,
    /// Total execution time across all runs
    pub total_execution_time: Duration,
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Last execution time
    pub last_execution: Option<DateTime<Utc>>,
    /// Next scheduled execution time
    pub next_execution: Option<DateTime<Utc>>,
}

/// Type for the task's asynchronous callable function
pub type TaskFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// A schedulable task that can be executed asynchronously
pub struct Task {
    /// Unique identifier for the task
    id: String,
    /// Display name for the task
    name: String,
    /// The cron schedule for this task (if scheduled)
    schedule: Option<CronSchedule>,
    /// The function to execute
    function: TaskFn,
    /// Configuration for execution
    config: TaskConfig,
    /// Current status of the task
    status: Arc<Mutex<TaskStatus>>,
    /// Statistics about execution
    stats: Arc<Mutex<TaskStats>>,
}

impl Task {
    /// Create a new task with a function to execute
    pub fn new<F, Fut>(func: F) -> Self 
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let task_id = Uuid::new_v4().to_string();
        
        Task {
            id: task_id.clone(),
            name: format!("task_{}", task_id[..8].to_string()),
            schedule: None,
            function: Box::new(move || Box::pin(func())),
            config: TaskConfig::default(),
            status: Arc::new(Mutex::new(TaskStatus::Idle)),
            stats: Arc::new(Mutex::new(TaskStats::default())),
        }
    }
    
    /// Get the task's unique ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Set a display name for the task
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    
    /// Set the configuration for the task
    pub fn with_config(mut self, config: TaskConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Set the cron schedule for the task
    pub fn with_schedule(mut self, cron_expr: &str) -> Result<Self> {
        self.schedule = Some(CronSchedule::new(cron_expr)?);
        Ok(self)
    }
    
    /// Get the current status of the task
    pub async fn status(&self) -> TaskStatus {
        *self.status.lock().await
    }
    
    /// Get the task statistics
    pub async fn stats(&self) -> TaskStats {
        self.stats.lock().await.clone()
    }
    
    /// Update the next execution time based on the schedule
    pub async fn update_next_execution(&self) {
        if let Some(schedule) = &self.schedule {
            let mut stats = self.stats.lock().await;
            stats.next_execution = schedule.next_execution(Utc::now());
        }
    }
    
    /// Execute the task with retry logic and timeout handling
    pub async fn execute(&self) -> Result<()> {
        info!("Executing task: {}", self.name);
        
        // Update status to running
        {
            let mut status = self.status.lock().await;
            *status = TaskStatus::Running;
        }
        
        let start_time = Utc::now();
        let mut result = self.execute_once().await;
        let mut retries = 0;
        
        // Handle retries if needed
        while result.is_err() && retries < self.config.max_retries {
            retries += 1;
            warn!("Task {} failed, retrying ({}/{}): {:?}", 
                 self.name, retries, self.config.max_retries, result);
            
            // Wait before retrying
            tokio::time::sleep(self.config.retry_delay).await;
            
            // Try again
            result = self.execute_once().await;
        }
        
        // Calculate execution duration
        let duration = Utc::now()
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
        
        // Update statistics
        {
            let mut stats = self.stats.lock().await;
            stats.executions += 1;
            stats.last_execution = Some(start_time);
            stats.total_execution_time += duration;
            
            if stats.executions > 0 {
                stats.avg_execution_time = Duration::from_nanos(
                    (stats.total_execution_time.as_nanos() / stats.executions as u128) as u64
                );
            }
            
            // Update success/failure counts based on result
            if result.is_ok() {
                stats.successes += 1;
            } else {
                stats.failures += 1;
            }
        }
        
        // Update status based on result
        {
            let mut status = self.status.lock().await;
            *status = if result.is_ok() {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
        }
        
        // Update next execution time
        self.update_next_execution().await;
        
        // Return the result or error
        if result.is_err() {
            error!("Task {} failed after {} retries: {:?}", 
                  self.name, retries, result);
        } else {
            debug!("Task {} completed successfully", self.name);
        }
        
        result
    }
    
    /// Execute the task once with timeout handling
    async fn execute_once(&self) -> Result<()> {
        let func = &self.function;
        
        // Execute with timeout if configured
        match self.config.timeout {
            Some(timeout_duration) => {
                match timeout(timeout_duration, func()).await {
                    Ok(result) => result,
                    Err(_) => Err(TasklineError::TaskTimeout(format!(
                        "Task '{}' timed out after {:?}", self.name, timeout_duration
                    ))),
                }
            },
            None => func().await,
        }
    }
    
    /// Pause the task so it won't be executed by the scheduler
    pub async fn pause(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if *status != TaskStatus::Running {
            *status = TaskStatus::Paused;
            Ok(())
        } else {
            Err(TasklineError::TaskExecutionError(
                format!("Cannot pause task '{}' while it is running", self.name)
            ))
        }
    }
    
    /// Resume a paused task
    pub async fn resume(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if *status == TaskStatus::Paused {
            *status = TaskStatus::Idle;
            Ok(())
        } else {
            Err(TasklineError::TaskExecutionError(
                format!("Cannot resume task '{}' as it is not paused", self.name)
            ))
        }
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("schedule", &self.schedule)
            .field("config", &self.config)
            .finish()
    }
}