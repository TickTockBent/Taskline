//! Task management and execution.
//!
//! This module provides the [`Task`] type and related functionality for creating
//! and managing schedulable asynchronous tasks.

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

/// Represents the current execution status of a task.
///
/// Tasks transition through various states during their lifecycle:
/// - Start as `Idle`
/// - Move to `Running` during execution
/// - End as `Completed` or `Failed`
/// - Can be manually set to `Paused`
///
/// # Examples
///
/// ```
/// use taskline::TaskStatus;
///
/// let status = TaskStatus::Idle;
/// assert_eq!(status.to_string(), "idle");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is idle, waiting for its next scheduled execution.
    Idle,
    /// Task is currently executing.
    Running,
    /// Task completed its last execution successfully.
    Completed,
    /// Task failed during its last execution.
    Failed,
    /// Task is paused and will not be executed by the scheduler.
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

/// Configuration options for task execution behavior.
///
/// `TaskConfig` allows fine-grained control over how tasks are executed,
/// including timeout settings, retry behavior, and error handling.
///
/// # Examples
///
/// ```
/// use taskline::TaskConfig;
/// use std::time::Duration;
///
/// let config = TaskConfig {
///     timeout: Some(Duration::from_secs(30)),
///     max_retries: 5,
///     retry_delay: Duration::from_secs(10),
///     fail_scheduler_on_error: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Maximum execution time before the task is timed out.
    ///
    /// If `Some`, the task will be cancelled if it exceeds this duration.
    /// If `None`, the task can run indefinitely.
    pub timeout: Option<Duration>,

    /// Maximum number of retries on failure.
    ///
    /// If a task fails, it will be retried up to this many times before
    /// giving up. Set to 0 for no retries.
    pub max_retries: u32,

    /// Delay between retry attempts.
    ///
    /// After a task fails, this duration will elapse before the next retry.
    pub retry_delay: Duration,

    /// Whether a permanent task failure should stop the scheduler.
    ///
    /// If `true`, the scheduler will stop if this task fails after all retries.
    /// If `false`, the scheduler continues running other tasks.
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

/// Statistics tracking task execution history and performance.
///
/// `TaskStats` provides detailed metrics about a task's execution history,
/// including success/failure rates, timing information, and scheduling details.
///
/// # Examples
///
/// ```no_run
/// # use taskline::{Task, TaskStats};
/// # async fn example() {
/// let task = Task::new(|| async { Ok(()) });
/// task.execute().await.unwrap();
///
/// let stats: TaskStats = task.stats().await;
/// println!("Executions: {}", stats.executions);
/// println!("Success rate: {:.2}%",
///     (stats.successes as f64 / stats.executions as f64) * 100.0);
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    /// Total number of times the task has been executed.
    pub executions: u64,

    /// Number of successful executions.
    pub successes: u64,

    /// Number of failed executions (including retries).
    pub failures: u64,

    /// Cumulative execution time across all runs.
    pub total_execution_time: Duration,

    /// Average execution time per run.
    pub avg_execution_time: Duration,

    /// Timestamp of the most recent execution.
    pub last_execution: Option<DateTime<Utc>>,

    /// Timestamp of the next scheduled execution.
    pub next_execution: Option<DateTime<Utc>>,
}

/// Type alias for the task's asynchronous callable function.
///
/// This represents a function that returns a future which produces a `Result<()>`.
pub type TaskFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// A schedulable asynchronous task.
///
/// `Task` represents a unit of work that can be executed on a schedule or manually.
/// Tasks support retry logic, timeouts, statistics tracking, and pause/resume functionality.
///
/// # Examples
///
/// ## Basic Task
///
/// ```
/// use taskline::Task;
///
/// let task = Task::new(|| async {
///     println!("Task executing!");
///     Ok(())
/// }).with_name("My Task");
/// ```
///
/// ## Task with Configuration
///
/// ```
/// use taskline::{Task, TaskConfig};
/// use std::time::Duration;
///
/// let task = Task::new(|| async {
///     // Your task logic
///     Ok(())
/// })
/// .with_name("Configured Task")
/// .with_config(TaskConfig {
///     timeout: Some(Duration::from_secs(30)),
///     max_retries: 3,
///     retry_delay: Duration::from_secs(5),
///     fail_scheduler_on_error: false,
/// });
/// ```
///
/// ## Task with State
///
/// ```
/// use taskline::Task;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU32, Ordering};
///
/// let counter = Arc::new(AtomicU32::new(0));
/// let counter_clone = Arc::clone(&counter);
///
/// let task = Task::new(move || {
///     let counter = Arc::clone(&counter_clone);
///     async move {
///         counter.fetch_add(1, Ordering::SeqCst);
///         Ok(())
///     }
/// });
/// ```
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
    /// Creates a new task with the given async function.
    ///
    /// The function will be called each time the task is executed. It should return
    /// a future that produces a `Result<()>`.
    ///
    /// # Arguments
    ///
    /// * `func` - An async function or closure that returns `Result<()>`
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::Task;
    ///
    /// // Simple task
    /// let task = Task::new(|| async {
    ///     println!("Hello from task!");
    ///     Ok(())
    /// });
    ///
    /// // Task with error handling
    /// let task = Task::new(|| async {
    ///     // Do work...
    ///     if false {
    ///         return Err("Something went wrong".into());
    ///     }
    ///     Ok(())
    /// });
    /// ```
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


    /// Returns the task's unique identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::Task;
    ///
    /// let task = Task::new(|| async { Ok(()) });
    /// println!("Task ID: {}", task.id());
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Sets a display name for the task (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `name` - The name to assign to this task
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::Task;
    ///
    /// let task = Task::new(|| async { Ok(()) })
    ///     .with_name("Daily Backup");
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the configuration for the task (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for this task
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::{Task, TaskConfig};
    /// use std::time::Duration;
    ///
    /// let task = Task::new(|| async { Ok(()) })
    ///     .with_config(TaskConfig {
    ///         timeout: Some(Duration::from_secs(30)),
    ///         max_retries: 5,
    ///         retry_delay: Duration::from_secs(2),
    ///         fail_scheduler_on_error: false,
    ///     });
    /// ```
    pub fn with_config(mut self, config: TaskConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the cron schedule for the task (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `cron_expr` - A cron expression string (e.g., "0 * * * *")
    ///
    /// # Returns
    ///
    /// Returns `Ok(Task)` if the cron expression is valid, or
    /// `Err(TasklineError)` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::Task;
    ///
    /// let task = Task::new(|| async { Ok(()) })
    ///     .with_schedule("*/5 * * * *") // Every 5 minutes
    ///     .unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`TasklineError::CronParseError`] if the cron expression is invalid.
    pub fn with_schedule(mut self, cron_expr: &str) -> Result<Self> {
        self.schedule = Some(CronSchedule::new(cron_expr)?);
        Ok(self)
    }

    /// Returns the current status of the task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use taskline::Task;
    /// # async fn example() {
    /// let task = Task::new(|| async { Ok(()) });
    /// let status = task.status().await;
    /// println!("Task status: {}", status);
    /// # }
    /// ```
    pub async fn status(&self) -> TaskStatus {
        *self.status.lock().await
    }

    /// Returns a snapshot of the task's execution statistics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use taskline::Task;
    /// # async fn example() {
    /// let task = Task::new(|| async { Ok(()) });
    /// task.execute().await.unwrap();
    ///
    /// let stats = task.stats().await;
    /// println!("Executions: {}, Successes: {}", stats.executions, stats.successes);
    /// # }
    /// ```
    pub async fn stats(&self) -> TaskStats {
        self.stats.lock().await.clone()
    }


    /// Updates the next execution time based on the task's schedule.
    ///
    /// This is typically called automatically by the scheduler, but can be
    /// called manually if needed.
    pub async fn update_next_execution(&self) {
        if let Some(schedule) = &self.schedule {
            let mut stats = self.stats.lock().await;
            stats.next_execution = schedule.next_execution(Utc::now());
        }
    }

    /// Executes the task with retry logic and timeout handling.
    ///
    /// This method runs the task's function, applies the configured timeout,
    /// handles retries on failure, and updates statistics.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the task completed successfully (possibly after retries),
    /// or `Err(TasklineError)` if it failed after all retry attempts or timed out.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use taskline::Task;
    /// # async fn example() {
    /// let task = Task::new(|| async {
    ///     println!("Running task...");
    ///     Ok(())
    /// });
    ///
    /// match task.execute().await {
    ///     Ok(()) => println!("Task succeeded"),
    ///     Err(e) => eprintln!("Task failed: {}", e),
    /// }
    /// # }
    /// ```
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


    /// Pauses the task so it won't be executed by the scheduler.
    ///
    /// A paused task remains in the scheduler but won't be executed until
    /// it's resumed with [`Task::resume`].
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the task was successfully paused, or
    /// `Err(TasklineError)` if the task is currently running.
    ///
    /// # Errors
    ///
    /// Returns [`TasklineError::TaskExecutionError`] if the task is currently running.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use taskline::Task;
    /// # async fn example() {
    /// let task = Task::new(|| async { Ok(()) });
    /// task.pause().await.unwrap();
    /// # }
    /// ```
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


    /// Resumes a paused task.
    ///
    /// A resumed task will be executed by the scheduler according to its schedule.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the task was successfully resumed, or
    /// `Err(TasklineError)` if the task is not paused.
    ///
    /// # Errors
    ///
    /// Returns [`TasklineError::TaskExecutionError`] if the task is not paused.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use taskline::Task;
    /// # async fn example() {
    /// let task = Task::new(|| async { Ok(()) });
    /// task.pause().await.unwrap();
    /// // ... later ...
    /// task.resume().await.unwrap();
    /// # }
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_task_creation() {
        let task = Task::new(|| async { Ok(()) });

        assert!(!task.id().is_empty());
        assert_eq!(task.status().await, TaskStatus::Idle);

        let stats = task.stats().await;
        assert_eq!(stats.executions, 0);
        assert_eq!(stats.successes, 0);
        assert_eq!(stats.failures, 0);
    }

    #[tokio::test]
    async fn test_task_with_name() {
        let task = Task::new(|| async { Ok(()) })
            .with_name("Test Task");

        let debug_str = format!("{:?}", task);
        assert!(debug_str.contains("Test Task"));
    }

    #[tokio::test]
    async fn test_task_execution_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let task = Task::new(move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });

        let result = task.execute().await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let stats = task.stats().await;
        assert_eq!(stats.executions, 1);
        assert_eq!(stats.successes, 1);
        assert_eq!(stats.failures, 0);
    }

    #[tokio::test]
    async fn test_task_execution_failure() {
        let task = Task::new(|| async {
            Err(TasklineError::TaskExecutionError("intentional failure".to_string()))
        });

        let result = task.execute().await;
        assert!(result.is_err());

        let stats = task.stats().await;
        assert_eq!(stats.executions, 1);
        assert_eq!(stats.successes, 0);
        assert_eq!(stats.failures, 1);
    }

    #[tokio::test]
    async fn test_task_retry_logic() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let task = Task::new(move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(TasklineError::TaskExecutionError("retry me".to_string()))
                } else {
                    Ok(())
                }
            }
        })
        .with_config(TaskConfig {
            timeout: Some(Duration::from_secs(5)),
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            fail_scheduler_on_error: false,
        });

        let result = task.execute().await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let task = Task::new(|| async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(())
        })
        .with_config(TaskConfig {
            timeout: Some(Duration::from_millis(100)),
            max_retries: 0,
            retry_delay: Duration::from_secs(1),
            fail_scheduler_on_error: false,
        });

        let result = task.execute().await;
        assert!(result.is_err());

        if let Err(TasklineError::TaskTimeout(msg)) = result {
            assert!(msg.contains("timed out"));
        } else {
            panic!("Expected TaskTimeout error");
        }
    }

    #[tokio::test]
    async fn test_task_pause_resume() {
        let task = Task::new(|| async { Ok(()) });

        assert_eq!(task.status().await, TaskStatus::Idle);

        // Pause the task
        let result = task.pause().await;
        assert!(result.is_ok());
        assert_eq!(task.status().await, TaskStatus::Paused);

        // Resume the task
        let result = task.resume().await;
        assert!(result.is_ok());
        assert_eq!(task.status().await, TaskStatus::Idle);
    }

    #[tokio::test]
    async fn test_task_cannot_pause_while_running() {
        let task = Task::new(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        });

        // Start execution in background
        let task_clone = Arc::new(task);
        let task_for_pause = Arc::clone(&task_clone);

        tokio::spawn(async move {
            task_for_pause.execute().await
        });

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try to pause while running
        let result = task_clone.pause().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_with_schedule() {
        let task = Task::new(|| async { Ok(()) })
            .with_schedule("0 0 * * *");

        assert!(task.is_ok());

        let task = task.unwrap();
        let stats = task.stats().await;
        assert!(stats.next_execution.is_none());

        // Update next execution
        task.update_next_execution().await;
        let stats = task.stats().await;
        assert!(stats.next_execution.is_some());
    }

    #[tokio::test]
    async fn test_task_status_transitions() {
        let task = Task::new(|| async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        });

        assert_eq!(task.status().await, TaskStatus::Idle);

        let task = Arc::new(task);
        let task_clone = Arc::clone(&task);

        tokio::spawn(async move {
            task_clone.execute().await
        });

        // Give it time to start
        tokio::time::sleep(Duration::from_millis(10)).await;
        let status = task.status().await;
        assert!(status == TaskStatus::Running || status == TaskStatus::Completed);

        // Wait for completion
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(task.status().await, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_task_config_default() {
        let config = TaskConfig::default();

        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_secs(5));
        assert!(!config.fail_scheduler_on_error);
    }

    #[tokio::test]
    async fn test_task_stats_tracking() {
        let task = Task::new(|| async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        });

        // Execute multiple times
        for _ in 0..3 {
            task.execute().await.unwrap();
        }

        let stats = task.stats().await;
        assert_eq!(stats.executions, 3);
        assert_eq!(stats.successes, 3);
        assert!(stats.total_execution_time > Duration::from_millis(0));
        assert!(stats.avg_execution_time > Duration::from_millis(0));
        assert!(stats.last_execution.is_some());
    }
}