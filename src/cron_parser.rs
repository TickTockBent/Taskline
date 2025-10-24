//! Internal cron expression parsing and scheduling.
//!
//! This module provides functionality for parsing cron expressions and calculating
//! the next execution times for scheduled tasks.

use cron::Schedule;
use std::str::FromStr;
use chrono::{DateTime, Utc, Duration};
use log::{debug, trace};

use crate::errors::TasklineError;

/// A parsed cron expression that can calculate next execution times.
///
/// `CronSchedule` wraps a cron expression and provides methods to determine
/// when a task should be executed based on that schedule.
///
/// # Cron Expression Format
///
/// Uses standard cron syntax: `* * * * *`
/// - Minute (0-59)
/// - Hour (0-23)
/// - Day of month (1-31)
/// - Month (1-12)
/// - Day of week (0-6, Sunday = 0)
///
/// # Examples
///
/// ```ignore
/// use crate::cron_parser::CronSchedule;
/// use chrono::Utc;
///
/// let schedule = CronSchedule::new("0 12 * * *").unwrap();
/// let next = schedule.next_execution(Utc::now());
/// ```
#[derive(Debug, Clone)]
pub struct CronSchedule {
    /// The original cron expression string
    expression: String,
    /// Parsed schedule from the cron crate
    schedule: Schedule,
}

impl CronSchedule {
    /// Creates a new `CronSchedule` from a cron expression string.
    ///
    /// # Arguments
    ///
    /// * `expression` - A cron expression string (e.g., "0 * * * *")
    ///
    /// # Returns
    ///
    /// Returns `Ok(CronSchedule)` if the expression is valid, or
    /// `Err(TasklineError::CronParseError)` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let schedule = CronSchedule::new("*/5 * * * *")?; // Every 5 minutes
    /// let schedule = CronSchedule::new("0 0 * * MON")?; // Mondays at midnight
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`TasklineError::CronParseError`] if the expression is invalid.
    pub fn new(expression: &str) -> Result<Self, TasklineError> {
        debug!("Parsing cron expression: {}", expression);

        // The cron crate expects 6 or 7 fields (seconds minute hour day month dayofweek [year])
        // Convert traditional 5-field cron to 6-field by prepending "0" for seconds
        let cron_expr = if expression.split_whitespace().count() == 5 {
            format!("0 {}", expression)
        } else {
            expression.to_string()
        };

        let schedule = match Schedule::from_str(&cron_expr) {
            Ok(schedule) => schedule,
            Err(e) => {
                return Err(TasklineError::CronParseError(
                    format!("Invalid cron expression '{}': {}", expression, e)
                ));
            }
        };

        Ok(CronSchedule {
            expression: expression.to_string(),
            schedule,
        })
    }

    /// Returns the original cron expression string.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let schedule = CronSchedule::new("0 * * * *")?;
    /// assert_eq!(schedule.expression(), "0 * * * *");
    /// ```
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Calculates the next execution time after the given time.
    ///
    /// # Arguments
    ///
    /// * `after` - The time after which to find the next execution
    ///
    /// # Returns
    ///
    /// Returns `Some(DateTime<Utc>)` if there is a next execution time,
    /// or `None` if the schedule has no future executions.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chrono::Utc;
    ///
    /// let schedule = CronSchedule::new("0 0 * * *")?;
    /// let next = schedule.next_execution(Utc::now());
    /// ```
    pub fn next_execution(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        trace!("Calculating next execution time after: {}", after);
        self.schedule.after(&after).next()
    }

    /// Calculates the duration until the next execution from now.
    ///
    /// # Returns
    ///
    /// Returns `Some(Duration)` representing the time until the next execution,
    /// or `None` if there are no future executions.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let schedule = CronSchedule::new("0 * * * *")?;
    /// if let Some(duration) = schedule.time_until_next() {
    ///     println!("Next run in {} seconds", duration.num_seconds());
    /// }
    /// ```
    pub fn time_until_next(&self) -> Option<Duration> {
        let now = Utc::now();
        match self.next_execution(now) {
            Some(next) => Some(next.signed_duration_since(now)),
            None => None,
        }
    }

    /// Checks if the schedule should execute at the given time.
    ///
    /// Returns `true` if the given time matches the cron schedule (within 1 second).
    ///
    /// # Arguments
    ///
    /// * `time` - The time to check
    ///
    /// # Returns
    ///
    /// `true` if the schedule matches the given time, `false` otherwise.
    pub fn should_execute_at(&self, time: DateTime<Utc>) -> bool {
        // Check if the next scheduled time after (time - 1 second) is within 1 second of time
        let check_from = time - chrono::Duration::seconds(1);
        if let Some(next) = self.schedule.after(&check_from).next() {
            // If the next time is within one second of the given time,
            // we should execute
            let diff = if next > time {
                next - time
            } else {
                time - next
            };
            diff < chrono::Duration::seconds(1)
        } else {
            false
        }
    }
}

/// Helper function to determine if the current time matches a cron schedule.
///
/// # Arguments
///
/// * `cron_expr` - A cron expression string to check
///
/// # Returns
///
/// Returns `Ok(true)` if the current time matches the schedule,
/// `Ok(false)` if it doesn't match, or `Err` if the expression is invalid.
///
/// # Errors
///
/// Returns [`TasklineError::CronParseError`] if the cron expression is invalid.
///
/// # Examples
///
/// ```ignore
/// if is_scheduled_now("0 * * * *")? {
///     println!("It's the top of the hour!");
/// }
/// ```
pub fn is_scheduled_now(cron_expr: &str) -> Result<bool, TasklineError> {
    let schedule = CronSchedule::new(cron_expr)?;
    Ok(schedule.should_execute_at(Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_parse_valid_expression() {
        let result = CronSchedule::new("* * * * *");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_invalid_expression() {
        let result = CronSchedule::new("invalid");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_next_execution() {
        let schedule = CronSchedule::new("0 0 * * *").unwrap(); // Daily at midnight
        let now = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        
        let next = schedule.next_execution(now).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap());
    }
    
    #[test]
    fn test_should_execute_at() {
        let schedule = CronSchedule::new("0 12 * * *").unwrap(); // Daily at noon
        
        // Should execute at noon
        let noon = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        assert!(schedule.should_execute_at(noon));
        
        // Should not execute at 12:01
        let after_noon = Utc.with_ymd_and_hms(2023, 1, 1, 12, 1, 0).unwrap();
        assert!(!schedule.should_execute_at(after_noon));
    }
}