#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(clippy::all)]
//! Allow dead code in examples
#![allow(dead_code)]
#![allow(unused_variables)]

//! Example demonstrating the enhanced features of Cronline
//!
//! This example showcases:
//! - Interval-based scheduling
//! - Task tags/labels
//! - Event bus subscription
//! - Task cancellation
//! - Auto-generated task names

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use cronline::{Scheduler, SchedulerEvent, Task};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init(); // Optional: enable with basic-logging feature

    println!("=== Cronline Enhanced Features Demo ===\n");

    let scheduler = Scheduler::new();

    // Subscribe to events
    let mut event_receiver = scheduler.event_bus().subscribe();

    // Spawn a task to handle events
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::SchedulerStarted { timestamp } => {
                    println!("📅 Scheduler started at {}", timestamp);
                }
                SchedulerEvent::TaskAdded { task_name, .. } => {
                    println!("➕ Task added: {}", task_name);
                }
                SchedulerEvent::TaskStarting { task_name, .. } => {
                    println!("▶️  Task starting: {}", task_name);
                }
                SchedulerEvent::TaskCompleted {
                    task_name,
                    duration_ms,
                    ..
                } => {
                    println!("✅ Task completed: {} ({}ms)", task_name, duration_ms);
                }
                SchedulerEvent::TaskFailed {
                    task_name, error, ..
                } => {
                    println!("❌ Task failed: {} - {}", task_name, error);
                }
                SchedulerEvent::TaskTimeoutWarning {
                    task_name,
                    percent_complete,
                    ..
                } => {
                    println!(
                        "⚠️  Task timeout warning: {} ({}%)",
                        task_name, percent_complete
                    );
                }
                SchedulerEvent::SchedulerStopped { uptime_seconds, .. } => {
                    println!("🛑 Scheduler stopped after {}s", uptime_seconds);
                    break;
                }
                _ => {}
            }
        }
    });

    // Example 1: Interval-based scheduling with auto-generated name
    println!("1. Creating interval-based task...");
    let counter1 = Arc::new(AtomicU32::new(0));
    let counter1_clone = Arc::clone(&counter1);

    let task1 = Task::new(move || {
        let counter = Arc::clone(&counter1_clone);
        async move {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            println!("  Interval task executed #{}", count + 1);
            Ok(())
        }
    })
    .with_interval(Duration::from_secs(2))
    .with_tags(&["demo", "interval"]);

    scheduler.add_task(task1).await?;

    // Example 2: Cron-based task with custom name and tags
    println!("2. Creating cron-based task with tags...");
    let task2 = Task::new(|| async {
        println!("  Cron task: Running backup simulation...");
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    })
    .with_name("Database Backup")
    .with_schedule("*/3 * * * *")? // Every 3 minutes (for demo purposes)
    .with_tags(&["backup", "database", "critical"]);

    scheduler.add_task(task2).await?;

    // Example 3: Task with timeout and warning
    println!("3. Creating task with timeout warning...");
    let task3 = Task::new(|| async {
        println!("  Long-running task started...");
        tokio::time::sleep(Duration::from_secs(8)).await;
        println!("  Long-running task finished!");
        Ok(())
    })
    .with_name("Long Running Analysis")
    .with_interval(Duration::from_secs(15))
    .with_tag("analytics");

    scheduler.add_task(task3).await?;

    // Example 4: Cancellable task
    println!("4. Creating cancellable task...");
    let task4 = Task::new(|| async {
        println!("  Cancellable task running...");
        for i in 1..=10 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("    Progress: {}%", i * 10);
        }
        Ok(())
    })
    .with_name("Cancellable Task")
    .with_interval(Duration::from_secs(20))
    .with_tag("demo");

    let task4_id = scheduler.add_task(task4).await?;

    println!("\n▶️  Starting scheduler...\n");
    scheduler.start().await?;

    // Let tasks run for a bit
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Demonstrate task filtering by tags
    println!("\n🔍 Finding tasks with 'demo' tag...");
    let demo_tasks = scheduler.tasks_with_tag("demo").await;
    println!("Found {} demo tasks", demo_tasks.len());

    // Demonstrate task cancellation
    println!("\n🚫 Cancelling task '{}'...", task4_id);
    if let Some(task) = scheduler.get_task(&task4_id).await {
        task.cancel().await;
    }

    // Let it run a bit more
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Show statistics for all tasks
    println!("\n📊 Task Statistics:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            let stats = task.stats().await;
            println!(
                "  {} - Executions: {}, Success: {}, Failed: {}",
                task.name(),
                stats.executions,
                stats.successes,
                stats.failures
            );
        }
    }

    println!("\n🛑 Stopping scheduler...\n");
    scheduler.stop().await?;

    // Give event handler time to process final events
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n✨ Demo completed!");

    Ok(())
}
