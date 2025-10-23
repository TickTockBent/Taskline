# Taskline

Taskline is a minimalist yet powerful cron-style task scheduler built with Rust. Designed for simplicity, performance, and ease of integration, Taskline enables developers to effortlessly manage scheduled asynchronous tasks within Rust applications.

[![CI](https://github.com/yourusername/taskline/workflows/CI/badge.svg)](https://github.com/yourusername/taskline/actions)
[![Coverage](https://codecov.io/gh/yourusername/taskline/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/taskline)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/taskline.svg)](https://crates.io/crates/taskline)
[![Documentation](https://docs.rs/taskline/badge.svg)](https://docs.rs/taskline)

## Purpose

Taskline addresses the common need among Rust developers for reliable task scheduling without the overhead of external cron utilities or complex frameworks. It's ideal for:

- Web services
- Automation tools
- Background job management
- Bots
- Microservices

## Core Features

- **Cron-like Scheduling**: Intuitive scheduling using cron syntax (`0 0 * * *`)
- **Interval-based Scheduling**: Simple interval-based task execution with `Task::with_interval()`
- **Async-Await Native**: First-class async task support leveraging Tokio
- **Task Tags/Labels**: Organize and filter tasks using tags
- **Graceful Cancellation**: Cancel running tasks cleanly
- **Timeout Warnings**: Get warned at 80% of timeout duration
- **Event Bus**: Subscribe to scheduler and task lifecycle events
- **Auto-generated Names**: Meaningful task names automatically generated from cron expressions
- **Robust Logging**: Built-in logging integration (`log`, `env_logger`, or `tracing`)
- **Fault-Tolerance & Retries**: Configurable error handling and automatic task retries
- **Simple & Modular**: Easy to include and extend in existing Rust codebases

## Installation

Add Taskline to your Cargo.toml:

```toml
[dependencies]
taskline = "0.1.0"
```

## Basic Usage

```rust
use taskline::{Scheduler, Task};
use tokio;

#[tokio::main]
async fn main() {
    // Create a new scheduler
    let scheduler = Scheduler::new();

    // Add a task that runs every day at 6am
    scheduler.add("0 6 * * *", Task::new(|| async {
        println!("Performing daily maintenance task...");
        // Task implementation here
        Ok(())
    })).unwrap();

    // Start the scheduler (this blocks until the scheduler is stopped)
    scheduler.run().await;
}
```

## Advanced Usage

```rust
use taskline::{Scheduler, Task, TaskConfig, SchedulerConfig};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a scheduler with custom configuration
    let scheduler = Scheduler::with_config(SchedulerConfig {
        check_interval: Duration::from_millis(100),
        continue_on_error: true,
        shutdown_grace_period: Duration::from_secs(10),
    });

    // Add a task with custom retry configuration
    scheduler.add("*/5 * * * *", Task::new(|| async {
        // Task that runs every 5 minutes
        // ...
        Ok(())
    })
    .with_name("Background Process")
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(60)),
        max_retries: 3,
        retry_delay: Duration::from_secs(5),
        fail_scheduler_on_error: false,
    })).unwrap();

    // Start the scheduler in the background
    scheduler.start().await.unwrap();

    // Do other work...

    // Stop the scheduler when done
    scheduler.stop().await.unwrap();
}
```

## Enhanced Features

### Interval-based Scheduling

```rust
// Create a task that runs every 5 minutes
let task = Task::new(|| async {
    println!("Running periodic task!");
    Ok(())
})
.with_interval(Duration::from_secs(300));

scheduler.add_task(task)?;
```

### Task Tags and Filtering

```rust
// Create tasks with tags
let backup_task = Task::new(|| async { Ok(()) })
    .with_tags(&["backup", "database", "critical"]);

let monitoring_task = Task::new(|| async { Ok(()) })
    .with_tag("monitoring");

scheduler.add_task(backup_task)?;
scheduler.add_task(monitoring_task)?;

// Filter tasks by tags
let critical_tasks = scheduler.tasks_with_tag("critical").await;
let backup_or_monitoring = scheduler.tasks_with_any_tag(&["backup", "monitoring"]).await;
let critical_backups = scheduler.tasks_with_all_tags(&["backup", "critical"]).await;
```

### Event Bus

```rust
// Subscribe to scheduler events
let mut events = scheduler.event_bus().subscribe();

tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            SchedulerEvent::TaskStarting { task_name, .. } => {
                println!("Task starting: {}", task_name);
            }
            SchedulerEvent::TaskCompleted { task_name, duration_ms, .. } => {
                println!("Task completed: {} ({}ms)", task_name, duration_ms);
            }
            SchedulerEvent::TaskTimeoutWarning { task_name, .. } => {
                println!("Task approaching timeout: {}", task_name);
            }
            _ => {}
        }
    }
});
```

### Task Cancellation

```rust
// Cancel a running task
if let Some(task) = scheduler.get_task(&task_id).await {
    task.cancel().await;
}
```

### Auto-generated Task Names

```rust
// Task names are automatically generated from cron expressions
let task = Task::new(|| async { Ok(()) })
    .with_schedule("0 * * * *")?;  // Automatically named "Hourly"

let task = Task::new(|| async { Ok(()) })
    .with_schedule("*/5 * * * *")?;  // Automatically named "Every 5 Minutes"

let task = Task::new(|| async { Ok(()) })
    .with_interval(Duration::from_secs(300));  // Automatically named "Every 5 Minutes"
```

## Task Management

```rust
// Get a list of all task IDs
let task_ids = scheduler.task_ids().await;

// Get a reference to a specific task
if let Some(task) = scheduler.get_task(&task_id).await {
    // Get task statistics
    let stats = task.stats().await;
    println!("Task executions: {}", stats.executions);

    // Manually execute a task
    task.execute().await.unwrap();

    // Pause a task
    task.pause().await.unwrap();

    // Resume a paused task
    task.resume().await.unwrap();

    // Cancel a running task
    task.cancel().await;
}

// Remove a task from the scheduler
scheduler.remove(&task_id).unwrap();
```

## Cron Expression Syntax

Taskline uses standard cron syntax: `* * * * *` (minute, hour, day of month, month, day of week)

Examples:
- `* * * * *` - Every minute
- `0 * * * *` - Every hour at minute 0
- `0 0 * * *` - Every day at midnight
- `0 12 * * MON-FRI` - Weekdays at noon
- `*/15 * * * *` - Every 15 minutes

## Recent Enhancements (v0.1.1)

- ✅ Interval-based scheduling with `Task::with_interval(Duration)`
- ✅ Task tags/labels for grouping and filtering
- ✅ Graceful task cancellation
- ✅ Task timeout warnings at 80% of timeout
- ✅ Scheduler event bus for task/scheduler events
- ✅ Auto-generated meaningful names from cron expressions

## Future Enhancements

- Task persistence with databases (SQLite, Redis)
- Interactive CLI for task management
- Web dashboard integration
- Task dependencies and workflows
- Distributed task scheduling

## License

This project is licensed under the MIT License - see the LICENSE file for details.
