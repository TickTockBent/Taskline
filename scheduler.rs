use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{self, Instant};

use crate::cron_parser::CronSchedule;
use crate::errors::TasklineError;
use crate::task::{Task, TaskStatus};
use crate::Result;

/// Configuration options for the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How frequently to check for tasks that need to be executed (in ms)
    pub check_interval: Duration,
    /// Whether to continue running when a task fails
    pub continue_on_error: bool,
    /// How long to wait for tasks to complete on shutdown
    pub shutdown_grace_period: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        SchedulerConfig {
            check_interval: Duration::from_millis(500),
            continue_on_error: true,
            shutdown_grace_period: Duration::from_secs(30),
        }
    }
}

/// The main scheduler that manages and executes tasks
pub struct Scheduler {
    /// Map of task IDs to tasks
    tasks: Arc<Mutex<HashMap<String, Arc<Task>>>>,
    /// Scheduler configuration
    config: SchedulerConfig,
    /// Handle to the scheduler's background process
    scheduler_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Whether the scheduler is running
    running: Arc<Mutex<bool>>,
    /// The time the scheduler was started
    start_time: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl Scheduler {
    /// Create a new scheduler with default configuration
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }
    
    /// Create a new scheduler with the specified configuration
    pub fn with_config(config: SchedulerConfig) -> Self {
        Scheduler {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            config,
            scheduler_handle: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Add a task to be executed according to a cron schedule
    pub fn add(&self, cron_expr: &str, task: Task) -> Result<String> {
        let task = task.with_schedule(cron_expr)?;
        let task_id = task.id().to_string();
        
        let mut tasks = self.tasks.blocking_lock();
        tasks.insert(task_id.clone(), Arc::new(task));
        
        debug!("Added task '{}' with schedule '{}'", task_id, cron_expr);
        Ok(task_id)
    }
    
    /// Add a pre-configured task to the scheduler
    pub fn add_task(&self, task: Task) -> Result<String> {
        let task_id = task.id().to_string();
        let task_arc = Arc::new(task);
        
        let mut tasks = self.tasks.blocking_lock();
        tasks.insert(task_id.clone(), task_arc);
        
        debug!("Added task '{}'", task_id);
        Ok(task_id)
    }
    
    /// Remove a task from the scheduler
    pub fn remove(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.blocking_lock();
        if tasks.remove(task_id).is_some() {
            debug!("Removed task '{}'", task_id);
            Ok(())
        } else {
            Err(TasklineError::SchedulerError(format!(
                "Task '{}' not found", task_id
            )))
        }
    }
    
    /// Get a reference to a task by ID
    pub async fn get_task(&self, task_id: &str) -> Option<Arc<Task>> {
        self.tasks.lock().await.get(task_id).cloned()
    }
    
    /// Get a list of all task IDs
    pub async fn task_ids(&self) -> Vec<String> {
        self.tasks.lock().await.keys().cloned().collect()
    }
    
    /// Start the scheduler and run it in the background
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(TasklineError::SchedulerError(
                "Scheduler is already running".to_string()
            ));
        }
        
        // Mark as running and record start time
        *running = true;
        *self.start_time.lock().await = Some(Utc::now());
        
        // Clone what we need for the background task
        let tasks = Arc::clone(&self.tasks);
        let config = self.config.clone();
        let running_arc = Arc::clone(&self.running);
        
        // Spawn the background scheduler task
        let handle = tokio::spawn(async move {
            Self::scheduler_loop(tasks, config, running_arc).await;
        });
        
        // Store the handle
        *self.scheduler_handle.lock().await = Some(handle);
        
        info!("Scheduler started with check interval of {:?}", self.config.check_interval);
        Ok(())
    }
    
    /// Run the scheduler in the foreground (blocks until stopped)
    pub async fn run(&self) -> Result<()> {
        self.start().await?;
        
        // Wait for the scheduler to be stopped
        let handle = {
            let mut handle_lock = self.scheduler_handle.lock().await;
            handle_lock.take()
        };
        
        if let Some(handle) = handle {
            handle.await.map_err(|e| {
                TasklineError::SchedulerError(format!("Scheduler task failed: {}", e))
            })?;
        }
        
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if !*running {
            return Err(TasklineError::SchedulerError(
                "Scheduler is not running".to_string()
            ));
        }
        
        // Mark as not running
        *running = false;
        
        // Get the handle
        let handle = {
            let mut handle_lock = self.scheduler_handle.lock().await;
            handle_lock.take()
        };
        
        // Wait for the scheduler to complete with timeout
        if let Some(handle) = handle {
            match tokio::time::timeout(self.config.shutdown_grace_period, handle).await {
                Ok(result) => {
                    result.map_err(|e| {
                        TasklineError::SchedulerError(format!("Scheduler task failed during shutdown: {}", e))
                    })?;
                },
                Err(_) => {
                    warn!("Scheduler did not shut down within grace period, forcing shutdown");
                }
            }
        }
        
        info!("Scheduler stopped");
        Ok(())
    }
    
    /// The main scheduler loop that checks for and executes due tasks
    async fn scheduler_loop(
        tasks: Arc<Mutex<HashMap<String, Arc<Task>>>>,
        config: SchedulerConfig,
        running: Arc<Mutex<bool>>
    ) {
        // Interval for periodic checking
        let mut interval = time::interval_at(
            Instant::now(), 
            config.check_interval
        );
        
        // Keep running until stopped
        while *running.lock().await {
            trace!("Scheduler tick - checking for due tasks");
            interval.tick().await;
            
            // Find and execute due tasks
            Self::execute_due_tasks(&tasks, &config).await;
        }
        
        info!("Scheduler loop terminated");
    }
    
    /// Check for and execute tasks that are due to run
    async fn execute_due_tasks(
        tasks: &Arc<Mutex<HashMap<String, Arc<Task>>>>,
        config: &SchedulerConfig
    ) {
        let now = Utc::now();
        let task_ids: Vec<(String, Arc<Task>)> = {
            let tasks_guard = tasks.lock().await;
            tasks_guard
                .iter()
                .map(|(id, task)| (id.clone(), Arc::clone(task)))
                .collect()
        };
        
        // Check each task to see if it's due
        for (task_id, task) in task_ids {
            // Skip if task isn't idle
            let status = task.status().await;
            if status != TaskStatus::Idle {
                continue;
            }
            
            // Get next execution time
            let stats = task.stats().await;
            let should_execute = if let Some(next_exec) = stats.next_execution {
                // If the next execution time is in the past or very near present, execute
                now >= next_exec || (next_exec - now).num_milliseconds() < 100
            } else {
                false
            };
            
            if should_execute {
                debug!("Task '{}' is due for execution", task_id);
                
                // Get a clone for the async block
                let task_clone = Arc::clone(&task);
                let config_clone = config.clone();
                
                // Spawn a new task to execute
                tokio::spawn(async move {
                    if let Err(e) = task_clone.execute().await {
                        error!("Task '{}' execution failed: {:?}", task_id, e);
                        
                        // If configured to stop on errors, stop the scheduler
                        if !config_clone.continue_on_error {
                            error!("Stopping scheduler due to task failure (continue_on_error=false)");
                            // Note: We can't directly stop the scheduler here,
                            // but the main app can check task results and stop if needed
                        }
                    }
                });
            }
        }
    }
    
    /// Update the next execution time for all tasks
    pub async fn update_next_executions(&self) -> Result<()> {
        let tasks = self.tasks.lock().await;
        for task in tasks.values() {
            task.update_next_execution().await;
        }
        
        Ok(())
    }
    
    /// Pause a task by ID
    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        if let Some(task) = self.get_task(task_id).await {
            task.pause().await
        } else {
            Err(TasklineError::SchedulerError(format!(
                "Task '{}' not found", task_id
            )))
        }
    }
    
    /// Resume a paused task by ID
    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        if let Some(task) = self.get_task(task_id).await {
            task.resume().await
        } else {
            Err(TasklineError::SchedulerError(format!(
                "Task '{}' not found", task_id
            )))
        }
    }
    
    /// Get the uptime of the scheduler
    pub async fn uptime(&self) -> Option<chrono::Duration> {
        if let Some(start_time) = *self.start_time.lock().await {
            Some(Utc::now().signed_duration_since(start_time))
        } else {
            None
        }
    }
    
    /// Check if the scheduler is currently running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Try to ensure the scheduler is stopped when dropped
        if let Ok(running) = self.running.try_lock() {
            if *running {
                warn!("Scheduler dropped while still running!");
            }
        }
    }
}