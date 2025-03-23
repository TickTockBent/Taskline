# Taskline

Taskline is a minimalist yet powerful cron-style task scheduler built with Rust. Designed for simplicity, performance, and ease of integration, Taskline enables developers to effortlessly manage scheduled asynchronous tasks within Rust applications.

[![Crates.io](https://img.shields.io/crates/v/taskline.svg)](https://crates.io/crates/taskline)
[![Documentation](https://docs.rs/taskline/badge.svg)](https://docs.rs/taskline)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Purpose

Taskline addresses the common need among Rust developers for reliable task scheduling without the overhead of external cron utilities or complex frameworks. It's ideal for:

- Web services
- Automation tools
- Background job management
- Bots
- Microservices

## Core Features

- **Cron-like Scheduling**: Intuitive scheduling using cron syntax (`0 0 * * *`)
- **Async-Await Native**: First-class async task support leveraging Tokio
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

## Future Enhancements

- Task persistence with databases (SQLite, Redis)
- Interactive CLI for task management
- Web dashboard integration

## License

This project is licensed under the MIT License - see the LICENSE file for details.