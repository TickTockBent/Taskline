//! Error types for Cronline operations.
//!
//! This module defines all error types that can occur during scheduler and task operations.

use std::error::Error;
use std::fmt;
use std::io;

/// Represents all possible errors that can occur in Cronline operations.
///
/// This error type covers all failure scenarios including cron parsing errors,
/// task execution failures, configuration issues, and scheduler management errors.
///
/// # Examples
///
/// ```
/// use cronline::{CronlineError, Result};
///
/// fn parse_cron(expr: &str) -> Result<()> {
///     if expr.is_empty() {
///         return Err(CronlineError::CronParseError("Empty expression".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum CronlineError {
    /// Error parsing a cron expression.
    ///
    /// This occurs when an invalid cron expression is provided to the scheduler.
    ///
    /// # Examples
    ///
    /// ```
    /// use cronline::CronlineError;
    ///
    /// let error = CronlineError::CronParseError("Invalid syntax".to_string());
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
    /// use cronline::CronlineError;
    ///
    /// let error = CronlineError::TaskExecutionError("Database connection failed".to_string());
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
    /// use cronline::CronlineError;
    ///
    /// let error = CronlineError::ConfigError("Invalid timeout value".to_string());
    /// ```
    ConfigError(String),

    /// Task execution exceeded timeout.
    ///
    /// This occurs when a task runs longer than its configured timeout duration.
    ///
    /// # Examples
    ///
    /// ```
    /// use cronline::CronlineError;
    ///
    /// let error = CronlineError::TaskTimeout("Task exceeded 30s timeout".to_string());
    /// ```
    TaskTimeout(String),

    /// Scheduler management error.
    ///
    /// This occurs during scheduler lifecycle operations like start, stop, or task management.
    ///
    /// # Examples
    ///
    /// ```
    /// use cronline::CronlineError;
    ///
    /// let error = CronlineError::SchedulerError("Already running".to_string());
    /// ```
    SchedulerError(String),
}

impl fmt::Display for CronlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CronlineError::CronParseError(msg) => write!(f, "Cron parsing error: {}", msg),
            CronlineError::TaskExecutionError(msg) => write!(f, "Task execution error: {}", msg),
            CronlineError::IoError(err) => write!(f, "IO error: {}", err),
            CronlineError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CronlineError::TaskTimeout(msg) => write!(f, "Task timeout: {}", msg),
            CronlineError::SchedulerError(msg) => write!(f, "Scheduler error: {}", msg),
        }
    }
}

impl Error for CronlineError {}

impl From<io::Error> for CronlineError {
    fn from(error: io::Error) -> Self {
        CronlineError::IoError(error)
    }
}

impl From<String> for CronlineError {
    fn from(error: String) -> Self {
        CronlineError::TaskExecutionError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let error = CronlineError::CronParseError("invalid cron".to_string());
        assert_eq!(error.to_string(), "Cron parsing error: invalid cron");

        let error = CronlineError::TaskExecutionError("task failed".to_string());
        assert_eq!(error.to_string(), "Task execution error: task failed");

        let error = CronlineError::ConfigError("bad config".to_string());
        assert_eq!(error.to_string(), "Configuration error: bad config");

        let error = CronlineError::TaskTimeout("timeout".to_string());
        assert_eq!(error.to_string(), "Task timeout: timeout");

        let error = CronlineError::SchedulerError("scheduler issue".to_string());
        assert_eq!(error.to_string(), "Scheduler error: scheduler issue");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cronline_error: CronlineError = io_error.into();

        match cronline_error {
            CronlineError::IoError(_) => (),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_string_conversion() {
        let error: CronlineError = "test error".to_string().into();

        match error {
            CronlineError::TaskExecutionError(msg) => {
                assert_eq!(msg, "test error");
            }
            _ => panic!("Expected TaskExecutionError variant"),
        }
    }

    #[test]
    fn test_error_trait() {
        let error = CronlineError::CronParseError("test".to_string());
        let _error_trait: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_debug_format() {
        let error = CronlineError::CronParseError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("CronParseError"));
    }
}
