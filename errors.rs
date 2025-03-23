use std::fmt;
use std::error::Error;
use std::io;

/// Represents all possible errors that can occur in Taskline
#[derive(Debug)]
pub enum TasklineError {
    /// Error parsing a cron expression
    CronParseError(String),
    
    /// Error during task execution
    TaskExecutionError(String),
    
    /// IO error during operation
    IoError(io::Error),
    
    /// Invalid configuration
    ConfigError(String),
    
    /// Task timeout
    TaskTimeout(String),
    
    /// Scheduler error
    SchedulerError(String),
}

impl fmt::Display for TasklineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TasklineError::CronParseError(msg) => write!(f, "Cron parsing error: {}", msg),
            TasklineError::TaskExecutionError(msg) => write!(f, "Task execution error: {}", msg),
            TasklineError::IoError(err) => write!(f, "IO error: {}", err),
            TasklineError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            TasklineError::TaskTimeout(msg) => write!(f, "Task timeout: {}", msg),
            TasklineError::SchedulerError(msg) => write!(f, "Scheduler error: {}", msg),
        }
    }
}

impl Error for TasklineError {}

impl From<io::Error> for TasklineError {
    fn from(error: io::Error) -> Self {
        TasklineError::IoError(error)
    }
}

impl From<String> for TasklineError {
    fn from(error: String) -> Self {
        TasklineError::TaskExecutionError(error)
    }
}