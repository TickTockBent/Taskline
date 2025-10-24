//! Error types for Taskline operations.
//!
//! This module defines all error types that can occur during scheduler and task operations.

use std::error::Error;
use std::fmt;
use std::io;

/// Represents all possible errors that can occur in Taskline operations.
///
/// This error type covers all failure scenarios including cron parsing errors,
/// task execution failures, configuration issues, and scheduler management errors.
///
/// # Examples
///
/// ```
/// use taskline::{TasklineError, Result};
///
/// fn parse_cron(expr: &str) -> Result<()> {
///     if expr.is_empty() {
///         return Err(TasklineError::CronParseError("Empty expression".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum TasklineError {
    /// Error parsing a cron expression.
    ///
    /// This occurs when an invalid cron expression is provided to the scheduler.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::TasklineError;
    ///
    /// let error = TasklineError::CronParseError("Invalid syntax".to_string());
    /// assert_eq!(error.to_string(), "Cron parsing error: Invalid syntax");
    /// ```
    CronParseError(String),

    /// Error during task execution.
    ///
    /// This occurs when a task's function returns an error during execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::TasklineError;
    ///
    /// let error = TasklineError::TaskExecutionError("Database connection failed".to_string());
    /// ```
    TaskExecutionError(String),

    /// IO error during operation.
    ///
    /// This wraps standard library IO errors that may occur during file operations
    /// or other IO-related tasks.
    IoError(io::Error),

    /// Invalid configuration provided.
    ///
    /// This occurs when scheduler or task configuration values are invalid or incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::TasklineError;
    ///
    /// let error = TasklineError::ConfigError("Invalid timeout value".to_string());
    /// ```
    ConfigError(String),

    /// Task execution exceeded timeout.
    ///
    /// This occurs when a task runs longer than its configured timeout duration.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::TasklineError;
    ///
    /// let error = TasklineError::TaskTimeout("Task exceeded 30s timeout".to_string());
    /// ```
    TaskTimeout(String),

    /// Scheduler management error.
    ///
    /// This occurs during scheduler lifecycle operations like start, stop, or task management.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::TasklineError;
    ///
    /// let error = TasklineError::SchedulerError("Already running".to_string());
    /// ```
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
            }
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
