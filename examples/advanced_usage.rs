#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(clippy::all)]
//! Allow dead code in examples
#![allow(dead_code)]
#![allow(unused_variables)]

// use env_logger::Builder;
// use log::LevelFilter;
use cronline::{Result as CronlineResult, Scheduler, SchedulerConfig, Task, TaskConfig};
use log::{error, info, warn};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// A shared counter to demonstrate task state sharing
struct Counter {
    value: Mutex<u32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            value: Mutex::new(0),
        }
    }

    async fn increment(&self) -> u32 {
        let mut value = self.value.lock().await;
        *value += 1;
        *value
    }

    async fn get(&self) -> u32 {
        *self.value.lock().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger with more verbose output
    // Builder::new().filter_level(LevelFilter::Debug).init();

    info!("Starting advanced scheduler example");

    // Create a shared counter for our tasks
    let counter = Arc::new(Counter::new());

    // Create a custom scheduler configuration
    let config = SchedulerConfig {
        check_interval: Duration::from_millis(200), // Check more frequently
        continue_on_error: true,                    // Continue even if tasks fail
        shutdown_grace_period: Duration::from_secs(5), // Allow 5 seconds for graceful shutdown
    };

    // Create a scheduler with custom config
    let scheduler = Scheduler::with_config(config);

    // Add a task with custom retry configuration
    let counter_clone = Arc::clone(&counter);
    scheduler
        .add(
            "*/2 * * * *",
            Task::new(move || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    info!("Running counter increment task");
                    let new_value = counter.increment().await;
                    info!("Counter incremented to: {}", new_value);
                    Ok(())
                }
            })
            .with_name("Counter Incrementer")
            .with_config(TaskConfig {
                timeout: Some(Duration::from_secs(30)),
                max_retries: 5,
                retry_delay: Duration::from_secs(1),
                fail_scheduler_on_error: false,
            }),
        )
        .await?;

    // Add a task that monitors the counter
    let counter_clone = Arc::clone(&counter);
    scheduler
        .add(
            "* * * * *",
            Task::new(move || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    let value = counter.get().await;
                    info!("Counter monitor: current value = {}", value);

                    if value > 10 {
                        warn!("Counter exceeded threshold of 10!");
                    }

                    Ok(())
                }
            })
            .with_name("Counter Monitor"),
        )
        .await?;

    // Add a task that sometimes fails to demonstrate error handling
    scheduler
        .add(
            "*/3 * * * *",
            Task::new(|| async {
                info!("Running potentially failing task");

                // Simulate work
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Randomly succeed or fail
                let random_value = rand::random::<f32>();

                if random_value < 0.3 {
                    // 30% chance of failure
                    error!("Task simulation failed with random value: {}", random_value);
                    Err("Simulated random failure".to_string().into())
                } else {
                    info!(
                        "Task simulation succeeded with random value: {}",
                        random_value
                    );
                    Ok(())
                }
            })
            .with_name("Flakey Task"),
        )
        .await?;

    // Add a task that demonstrates manual execution and pausing
    let mut_task = Task::new(|| async {
        info!("Running manual control task");
        Ok(())
    })
    .with_name("Manual Control Task");

    // We don't set a schedule, so this task won't run automatically
    let manual_task_id = scheduler.add_task(mut_task).await?;

    // Start the scheduler
    scheduler.start().await?;

    // Let it run for a bit
    info!("Scheduler running. Will demonstrate manual task control in 10 seconds...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Manually control a task
    info!("Getting manual task by ID: {}", manual_task_id);
    if let Some(task) = scheduler.get_task(&manual_task_id).await {
        info!("Executing manual task...");
        task.execute().await?;

        // Pause a scheduled task
        info!("Pausing the counter monitor task...");
        let task_ids = scheduler.task_ids().await;
        for id in &task_ids {
            if let Some(task) = scheduler.get_task(id).await {
                let stats = task.stats().await;
                info!(
                    "Task {} stats: {} executions, {} successes, {} failures",
                    id, stats.executions, stats.successes, stats.failures
                );
            }
        }
    } else {
        error!("Could not find manual task by ID");
    }

    // Run for a while longer
    info!("Continuing scheduler for 20 more seconds...");
    tokio::time::sleep(Duration::from_secs(20)).await;

    // Get scheduler uptime
    if let Some(uptime) = scheduler.uptime().await {
        info!(
            "Scheduler has been running for {} seconds",
            uptime.num_seconds()
        );
    }

    // Stop the scheduler
    info!("Stopping scheduler...");
    scheduler.stop().await?;

    info!(
        "Example complete. Final counter value: {}",
        counter.get().await
    );
    Ok(())
}

// Demonstrate custom task creation with error handling
async fn perform_database_maintenance() -> CronlineResult<()> {
    // Here you would typically connect to a database and perform operations
    info!("Performing database maintenance");

    // Simulate database connection
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Simulate some database work
    for i in 0..5 {
        info!("DB maintenance step {}/5", i + 1);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    info!("Database maintenance completed");
    Ok(())
}
