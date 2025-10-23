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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let error = TasklineError::CronParseError("invalid cron".to_string());
        assert_eq!(error.to_string(), "Cron parsing error: invalid cron");

        let error = TasklineError::TaskExecutionError("task failed".to_string());
        assert_eq!(error.to_string(), "Task execution error: task failed");

        let error = TasklineError::ConfigError("bad config".to_string());
        assert_eq!(error.to_string(), "Configuration error: bad config");

        let error = TasklineError::TaskTimeout("timeout".to_string());
        assert_eq!(error.to_string(), "Task timeout: timeout");

        let error = TasklineError::SchedulerError("scheduler issue".to_string());
        assert_eq!(error.to_string(), "Scheduler error: scheduler issue");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let taskline_error: TasklineError = io_error.into();

        match taskline_error {
            TasklineError::IoError(_) => (),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_string_conversion() {
        let error: TasklineError = "test error".to_string().into();

        match error {
            TasklineError::TaskExecutionError(msg) => {
                assert_eq!(msg, "test error");
            },
            _ => panic!("Expected TaskExecutionError variant"),
        }
    }

    #[test]
    fn test_error_trait() {
        let error = TasklineError::CronParseError("test".to_string());
        let _error_trait: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_debug_format() {
        let error = TasklineError::CronParseError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("CronParseError"));
    }
}