//! # Taskline: Lightweight Task Scheduling for Rust
//!
//! Taskline is a minimalist yet powerful cron-style task scheduler built with Rust.
//! Designed for simplicity, performance, and ease of integration, it enables developers
//! to effortlessly manage scheduled asynchronous tasks within Rust applications.
//!
//! ## Features
//!
//! - **Cron-like Scheduling**: Intuitive scheduling using standard cron syntax
//! - **Interval-based Scheduling**: Simple interval-based task execution
//! - **Async-Await Native**: First-class async task support leveraging Tokio
//! - **Robust Error Handling**: Configurable retries and timeout handling
//! - **Task Statistics**: Built-in execution tracking and metrics
//! - **Pause/Resume**: Dynamic task control without stopping the scheduler
//! - **Task Tags/Labels**: Organize and filter tasks using tags
//! - **Graceful Cancellation**: Cancel running tasks cleanly
//! - **Timeout Warnings**: Get warned at 80% of timeout duration
//! - **Event Bus**: Subscribe to scheduler and task lifecycle events
//! - **Auto-generated Names**: Meaningful task names from cron expressions
//! - **Thread-Safe**: All components are safe to share across threads
//!
//! ## Quick Start
//!
//! Add Taskline to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! taskline = "0.1.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ## Basic Example
//!
//! ```no_run
//! use taskline::{Scheduler, Task};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new scheduler
//!     let scheduler = Scheduler::new();
//!
//!     // Add a task that runs every hour at minute 0
//!     scheduler.add("0 * * * *", Task::new(|| async {
//!         println!("Running hourly task!");
//!         Ok(())
//!     }).with_name("Hourly Task"))?;
//!
//!     // Start the scheduler
//!     scheduler.start().await?;
//!
//!     // Let it run for a while
//!     tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
//!
//!     // Stop the scheduler gracefully
//!     scheduler.stop().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ```no_run
//! use taskline::{Scheduler, Task, TaskConfig, SchedulerConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create scheduler with custom configuration
//!     let scheduler = Scheduler::with_config(SchedulerConfig {
//!         check_interval: Duration::from_millis(100),
//!         continue_on_error: true,
//!         shutdown_grace_period: Duration::from_secs(10),
//!     });
//!
//!     // Subscribe to events
//!     let mut events = scheduler.event_bus().subscribe();
//!     tokio::spawn(async move {
//!         while let Ok(event) = events.recv().await {
//!             println!("Event: {:?}", event);
//!         }
//!     });
//!
//!     // Add task with custom configuration
//!     scheduler.add("*/5 * * * *", Task::new(|| async {
//!         // Your task logic here
//!         Ok(())
//!     })
//!     .with_name("Custom Task")
//!     .with_tags(&["backup", "critical"])
//!     .with_config(TaskConfig {
//!         timeout: Some(Duration::from_secs(30)),
//!         max_retries: 3,
//!         retry_delay: Duration::from_secs(5),
//!         fail_scheduler_on_error: false,
//!     }))?;
//!
//!     // Add interval-based task
//!     scheduler.add_task(Task::new(|| async {
//!         println!("Running every 5 minutes!");
//!         Ok(())
//!     })
//!     .with_interval(Duration::from_secs(300))
//!     .with_tag("monitoring"))?;
//!
//!     scheduler.start().await?;
//!
//!     // Filter tasks by tag
//!     let critical_tasks = scheduler.tasks_with_tag("critical").await;
//!     println!("Found {} critical tasks", critical_tasks.len());
//!
//!     // Do other work...
//!
//!     scheduler.stop().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Cron Expression Syntax
//!
//! Taskline uses standard cron syntax: `* * * * *` (minute, hour, day of month, month, day of week)
//!
//! Examples:
//! - `* * * * *` - Every minute
//! - `0 * * * *` - Every hour at minute 0
//! - `0 0 * * *` - Every day at midnight
//! - `0 12 * * MON-FRI` - Weekdays at noon
//! - `*/15 * * * *` - Every 15 minutes
//!
//! ## Main Components
//!
//! - [`Scheduler`] - Manages and executes tasks on a schedule
//! - [`Task`] - Represents a schedulable asynchronous task
//! - [`TasklineError`] - Error types for the library
//! - [`Result`] - Convenient result type alias

// Re-export the main components
pub use crate::scheduler::{Scheduler, SchedulerConfig};
pub use crate::task::{Task, TaskConfig, TaskStatus, TaskStats, ScheduleType};
pub use crate::errors::TasklineError;
pub use crate::events::{EventBus, SchedulerEvent};

// Main modules
pub mod scheduler;
pub mod task;
pub mod errors;
pub mod events;
mod cron_parser;

/// Convenient result type alias for Taskline operations.
///
/// This is equivalent to `std::result::Result<T, TasklineError>`.
///
/// # Examples
///
/// ```
/// use taskline::Result;
///
/// fn do_something() -> Result<()> {
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, TasklineError>;

/// The version of the Taskline library.
///
/// This is extracted from the `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");