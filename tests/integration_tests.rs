use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use taskline::{Scheduler, SchedulerConfig, Task, TaskConfig};

#[tokio::test]
async fn test_full_scheduler_lifecycle() {
    let scheduler = Scheduler::new();

    // Add multiple tasks
    let task1 = Task::new(|| async { Ok(()) }).with_name("Task 1");
    let task2 = Task::new(|| async { Ok(()) }).with_name("Task 2");

    let id1 = scheduler.add("* * * * *", task1).unwrap();
    let id2 = scheduler.add("*/5 * * * *", task2).unwrap();

    // Start scheduler
    scheduler.start().await.unwrap();
    assert!(scheduler.is_running().await);

    // Let it run briefly
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check tasks exist
    assert!(scheduler.get_task(&id1).await.is_some());
    assert!(scheduler.get_task(&id2).await.is_some());

    // Stop scheduler
    scheduler.stop().await.unwrap();
    assert!(!scheduler.is_running().await);
}

#[tokio::test]
async fn test_task_execution_with_state() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let task = Task::new(move || {
        let counter = Arc::clone(&counter_clone);
        async move {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    })
    .with_name("Counter Task");

    // Execute task multiple times
    for _ in 0..5 {
        task.execute().await.unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 5);

    let stats = task.stats().await;
    assert_eq!(stats.executions, 5);
    assert_eq!(stats.successes, 5);
    assert_eq!(stats.failures, 0);
}

#[tokio::test]
async fn test_error_handling_and_retries() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let task = Task::new(move || {
        let counter = Arc::clone(&counter_clone);
        async move {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err("Intentional failure".into())
            } else {
                Ok(())
            }
        }
    })
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(5)),
        max_retries: 5,
        retry_delay: Duration::from_millis(10),
        fail_scheduler_on_error: false,
    });

    let result = task.execute().await;
    assert!(result.is_ok());

    // Should have tried 3 times: initial + 2 retries
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_concurrent_task_execution() {
    let counter = Arc::new(AtomicU32::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let task = Task::new(move || {
            let counter = Arc::clone(&counter_clone);
            async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });

        let handle = tokio::spawn(async move {
            task.execute().await
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_task_pause_resume_workflow() {
    let scheduler = Scheduler::new();

    let task = Task::new(|| async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    })
    .with_name("Pausable Task");

    let task_id = scheduler.add("* * * * *", task).unwrap();

    // Pause task
    scheduler.pause_task(&task_id).await.unwrap();

    if let Some(task) = scheduler.get_task(&task_id).await {
        assert_eq!(task.status().await, taskline::task::TaskStatus::Paused);
    }

    // Resume task
    scheduler.resume_task(&task_id).await.unwrap();

    if let Some(task) = scheduler.get_task(&task_id).await {
        assert_eq!(task.status().await, taskline::task::TaskStatus::Idle);
    }
}

#[tokio::test]
async fn test_scheduler_with_custom_config() {
    let config = SchedulerConfig {
        check_interval: Duration::from_millis(50),
        continue_on_error: true,
        shutdown_grace_period: Duration::from_secs(5),
    };

    let scheduler = Scheduler::with_config(config);

    let task = Task::new(|| async { Ok(()) });
    scheduler.add("* * * * *", task).unwrap();

    scheduler.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_multiple_schedulers_independently() {
    let scheduler1 = Scheduler::new();
    let scheduler2 = Scheduler::new();

    let task1 = Task::new(|| async { Ok(()) }).with_name("Scheduler 1 Task");
    let task2 = Task::new(|| async { Ok(()) }).with_name("Scheduler 2 Task");

    scheduler1.add("* * * * *", task1).unwrap();
    scheduler2.add("* * * * *", task2).unwrap();

    scheduler1.start().await.unwrap();
    scheduler2.start().await.unwrap();

    assert!(scheduler1.is_running().await);
    assert!(scheduler2.is_running().await);

    scheduler1.stop().await.unwrap();
    scheduler2.stop().await.unwrap();

    assert!(!scheduler1.is_running().await);
    assert!(!scheduler2.is_running().await);
}

#[tokio::test]
async fn test_task_statistics_accuracy() {
    let task = Task::new(|| async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    })
    .with_name("Stats Task");

    // Execute the task once
    task.execute().await.unwrap();

    let stats = task.stats().await;
    assert_eq!(stats.executions, 1);
    assert_eq!(stats.successes, 1);
    assert_eq!(stats.failures, 0);
    assert!(stats.total_execution_time >= Duration::from_millis(50));
    assert!(stats.avg_execution_time >= Duration::from_millis(50));
    assert!(stats.last_execution.is_some());
}

#[tokio::test]
async fn test_task_timeout_enforcement() {
    let task = Task::new(|| async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    })
    .with_config(TaskConfig {
        timeout: Some(Duration::from_millis(100)),
        max_retries: 0,
        retry_delay: Duration::from_secs(1),
        fail_scheduler_on_error: false,
    });

    let result = task.execute().await;
    assert!(result.is_err());

    let stats = task.stats().await;
    assert_eq!(stats.failures, 1);
}

#[tokio::test]
async fn test_scheduler_uptime_tracking() {
    let scheduler = Scheduler::new();

    assert!(scheduler.uptime().await.is_none());

    scheduler.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let uptime = scheduler.uptime().await;
    assert!(uptime.is_some());

    let uptime_ms = uptime.unwrap().num_milliseconds();
    assert!(uptime_ms >= 200, "Uptime was {} ms, expected >= 200 ms", uptime_ms);

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_task_removal_from_scheduler() {
    let scheduler = Scheduler::new();

    let task1 = Task::new(|| async { Ok(()) }).with_name("Task 1");
    let task2 = Task::new(|| async { Ok(()) }).with_name("Task 2");

    let id1 = scheduler.add("* * * * *", task1).unwrap();
    let id2 = scheduler.add("* * * * *", task2).unwrap();

    assert_eq!(scheduler.task_ids().await.len(), 2);

    scheduler.remove(&id1).unwrap();
    assert_eq!(scheduler.task_ids().await.len(), 1);

    assert!(scheduler.get_task(&id1).await.is_none());
    assert!(scheduler.get_task(&id2).await.is_some());
}
