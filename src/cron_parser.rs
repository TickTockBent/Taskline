use cron::Schedule;
use std::str::FromStr;
use chrono::{DateTime, Utc, Duration};
use log::{debug, trace};

use crate::errors::TasklineError;

/// CronSchedule represents a parsed cron expression that can determine
/// the next execution time for a task.
#[derive(Debug, Clone)]
pub struct CronSchedule {
    expression: String,
    schedule: Schedule,
}

impl CronSchedule {
    /// Create a new CronSchedule from a cron expression string
    pub fn new(expression: &str) -> Result<Self, TasklineError> {
        debug!("Parsing cron expression: {}", expression);
        
        let schedule = match Schedule::from_str(expression) {
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
    
    /// Get the original cron expression
    pub fn expression(&self) -> &str {
        &self.expression
    }
    
    /// Calculate the next execution time after the given time
    pub fn next_execution(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        trace!("Calculating next execution time after: {}", after);
        self.schedule.after(&after).next()
    }
    
    /// Calculate the time until the next execution from now
    pub fn time_until_next(&self) -> Option<Duration> {
        let now = Utc::now();
        match self.next_execution(now) {
            Some(next) => Some(next.signed_duration_since(now)),
            None => None,
        }
    }
    
    /// Returns true if the schedule should execute at the given time
    pub fn should_execute_at(&self, time: DateTime<Utc>) -> bool {
        // Get the previous scheduled time before or at 'time'
        if let Some(prev) = self.schedule.before(&time).next() {
            // If the previous time is within one second of the given time,
            // we should execute
            (time - prev) < chrono::Duration::seconds(1)
        } else {
            false
        }
    }
}

/// Helper function to determine if the current time matches a cron schedule
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