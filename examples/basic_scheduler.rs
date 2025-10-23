use std::error::Error;
use log::{info, LevelFilter};
use env_logger::Builder;
use taskline::{Scheduler, Task};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
    
    info!("Starting basic scheduler example");
    
    // Create a new scheduler
    let scheduler = Scheduler::new();
    
    // Add a task that runs every minute
    scheduler.add("* * * * *", Task::new(|| async {
        info!("Executing task: every minute");
        Ok(())
    }).with_name("Every Minute Task"))?;
    
    // Add a task that runs every 5 minutes
    scheduler.add("*/5 * * * *", Task::new(|| async {
        info!("Executing task: every 5 minutes");
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }).with_name("Every 5 Minutes Task"))?;
    
    // Add a task that runs hourly
    scheduler.add("0 * * * *", Task::new(|| async {
        info!("Executing task: hourly task");
        
        // Simulate work with potential errors
        if rand::random::<bool>() {
            info!("Hourly task completed successfully");
            Ok(())
        } else {
            // This will trigger retries based on task configuration
            Err("Simulated random failure".into())
        }
    }).with_name("Hourly Task"))?;
    
    // Update the next execution time for all tasks
    scheduler.update_next_executions().await?;
    
    // Print scheduled tasks
    let task_ids = scheduler.task_ids().await;
    info!("Scheduled tasks: {}", task_ids.len());
    
    for task_id in &task_ids {
        if let Some(task) = scheduler.get_task(task_id).await {
            let stats = task.stats().await;
            if let Some(next_exec) = stats.next_execution {
                info!("Task '{}' next execution: {}", task_id, next_exec);
            }
        }
    }
    
    // Run the scheduler (this will block until the scheduler is stopped)
    info!("Starting scheduler...");

    // Start the scheduler in the background
    scheduler.start().await?;

    // Wait for 10 minutes
    info!("Scheduler running. Will run for 10 minutes...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;

    // Stop the scheduler
    info!("Stopping scheduler...");
    scheduler.stop().await?;

    info!("Scheduler stopped. Example complete.");
    Ok(())
}