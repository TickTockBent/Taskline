//! Real-world example: Taskline integration with Axum web framework
//!
//! This example demonstrates:
//! - Running a scheduler alongside an Axum web server
//! - API endpoints for managing scheduled tasks
//! - Health check tasks
//! - Scheduled data cleanup
//! - Metrics collection

use taskline::{Scheduler, Task, SchedulerEvent, TaskConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// Simulated application state
#[derive(Clone)]
struct AppState {
    scheduler: Arc<Scheduler>,
    metrics: Arc<RwLock<Metrics>>,
    database_pool: Arc<RwLock<DatabasePool>>,
}

#[derive(Default, Clone, Serialize)]
struct Metrics {
    requests_served: u64,
    active_users: u64,
    database_connections: u32,
    cache_hit_rate: f64,
    last_health_check: Option<String>,
}

// Simulated database pool
struct DatabasePool {
    active_connections: u32,
    idle_connections: u32,
}

impl DatabasePool {
    fn new() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 10,
        }
    }

    async fn cleanup_old_sessions(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        // Simulate cleaning up old database sessions
        println!("🗑️  Cleaning up old database sessions...");
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(42) // Simulated cleanup count
    }

    async fn optimize_tables(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate table optimization
        println!("⚡ Optimizing database tables...");
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    async fn backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate database backup
        println!("💾 Creating database backup...");
        tokio::time::sleep(Duration::from_millis(300)).await;
        Ok(())
    }
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    name: String,
    schedule: String,
    task_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 Starting Taskline + Axum Integration Demo\n");

    // Create scheduler
    let scheduler = Arc::new(Scheduler::new());

    // Create application state
    let metrics = Arc::new(RwLock::new(Metrics::default()));
    let database_pool = Arc::new(RwLock::new(DatabasePool::new()));

    let app_state = AppState {
        scheduler: scheduler.clone(),
        metrics: metrics.clone(),
        database_pool: database_pool.clone(),
    };

    // Subscribe to scheduler events
    let mut event_receiver = scheduler.event_bus().subscribe();
    let event_metrics = metrics.clone();

    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskCompleted { task_name, duration_ms, .. } => {
                    println!("✅ Task '{}' completed in {}ms", task_name, duration_ms);
                }
                SchedulerEvent::TaskFailed { task_name, error, .. } => {
                    eprintln!("❌ Task '{}' failed: {}", task_name, error);
                }
                _ => {}
            }
        }
    });

    // Setup scheduled tasks
    setup_scheduled_tasks(&app_state).await?;

    // Start the scheduler
    scheduler.start().await?;

    println!("\n📊 Scheduler started with the following tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            let stats = task.stats().await;
            println!("  - {} (Next: {:?})", task.name(), stats.next_execution);
        }
    }

    // In a real application, you would start the Axum server here:
    // let app = create_axum_router(app_state);
    // axum::Server::bind(&"0.0.0.0:3000".parse()?)
    //     .serve(app.into_make_service())
    //     .await?;

    // For this demo, just run for a while
    println!("\n▶️  Running tasks (press Ctrl+C to stop)...\n");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Cleanup
    println!("\n🛑 Shutting down...");
    scheduler.stop().await?;

    Ok(())
}

async fn setup_scheduled_tasks(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: Health check every 30 seconds
    let metrics_clone = state.metrics.clone();
    let health_check_task = Task::new(move || {
        let metrics = metrics_clone.clone();
        async move {
            let mut m = metrics.write().await;
            m.last_health_check = Some(chrono::Utc::now().to_rfc3339());

            println!("💚 Health check passed - System healthy");
            println!("   Active users: {}", m.active_users);
            println!("   Cache hit rate: {:.2}%", m.cache_hit_rate * 100.0);

            Ok(())
        }
    })
    .with_name("System Health Check")
    .with_interval(Duration::from_secs(30))
    .with_tags(&["monitoring", "health", "critical"]);

    state.scheduler.add_task(health_check_task).await?;

    // Task 2: Cleanup old database sessions every 5 minutes
    let db_pool_clone = state.database_pool.clone();
    let cleanup_task = Task::new(move || {
        let pool = db_pool_clone.clone();
        async move {
            let mut db = pool.write().await;
            match db.cleanup_old_sessions().await {
                Ok(count) => {
                    println!("🧹 Cleaned up {} old sessions", count);
                    Ok(())
                }
                Err(e) => Err(taskline::TasklineError::TaskExecutionError(e.to_string())),
            }
        }
    })
    .with_schedule("*/5 * * * *")?  // Every 5 minutes
    .with_tags(&["database", "maintenance"])
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(30)),
        max_retries: 3,
        retry_delay: Duration::from_secs(5),
        fail_scheduler_on_error: false,
    });

    state.scheduler.add_task(cleanup_task).await?;

    // Task 3: Database optimization - daily at 2 AM
    let db_pool_clone = state.database_pool.clone();
    let optimize_task = Task::new(move || {
        let pool = db_pool_clone.clone();
        async move {
            let mut db = pool.write().await;
            match db.optimize_tables().await {
                Ok(_) => {
                    println!("✨ Database tables optimized successfully");
                    Ok(())
                }
                Err(e) => Err(taskline::TasklineError::TaskExecutionError(e.to_string())),
            }
        }
    })
    .with_schedule("0 2 * * *")?  // Daily at 2 AM
    .with_tags(&["database", "optimization", "nightly"]);

    state.scheduler.add_task(optimize_task).await?;

    // Task 4: Database backup - daily at 3 AM
    let db_pool_clone = state.database_pool.clone();
    let backup_task = Task::new(move || {
        let pool = db_pool_clone.clone();
        async move {
            let db = pool.read().await;
            match db.backup().await {
                Ok(_) => {
                    println!("💾 Database backup completed successfully");
                    Ok(())
                }
                Err(e) => Err(taskline::TasklineError::TaskExecutionError(e.to_string())),
            }
        }
    })
    .with_schedule("0 3 * * *")?  // Daily at 3 AM
    .with_tags(&["database", "backup", "critical"]);

    state.scheduler.add_task(backup_task).await?;

    // Task 5: Metrics aggregation every minute
    let metrics_clone = state.metrics.clone();
    let metrics_task = Task::new(move || {
        let metrics = metrics_clone.clone();
        async move {
            let mut m = metrics.write().await;

            // Simulate updating metrics
            m.requests_served += rand::random::<u64>() % 100;
            m.active_users = rand::random::<u64>() % 1000;
            m.cache_hit_rate = (rand::random::<u64>() % 100) as f64 / 100.0;

            println!("📊 Metrics updated - Requests: {}, Users: {}",
                m.requests_served, m.active_users);

            Ok(())
        }
    })
    .with_interval(Duration::from_secs(60))
    .with_tags(&["metrics", "monitoring"]);

    state.scheduler.add_task(metrics_task).await?;

    // Task 6: Session cleanup warning at 80% timeout
    let warning_task = Task::new(|| async {
        println!("⏰ Long-running cleanup starting...");
        tokio::time::sleep(Duration::from_secs(8)).await;
        println!("✅ Long-running cleanup finished");
        Ok(())
    })
    .with_name("Long Cleanup Task")
    .with_interval(Duration::from_secs(20))
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(10)),
        max_retries: 1,
        retry_delay: Duration::from_secs(2),
        fail_scheduler_on_error: false,
    });

    state.scheduler.add_task(warning_task).await?;

    Ok(())
}

/*
// Example Axum router (commented out to avoid dependency in example)
fn create_axum_router(state: AppState) -> axum::Router {
    use axum::{
        routing::{get, post},
        Json, extract::State,
    };

    axum::Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id/cancel", post(cancel_task))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().await;
    Json(serde_json::json!({
        "status": "healthy",
        "last_check": metrics.last_health_check,
    }))
}

async fn get_metrics(State(state): State<AppState>) -> Json<Metrics> {
    let metrics = state.metrics.read().await;
    Json(metrics.clone())
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<String>> {
    let task_ids = state.scheduler.task_ids().await;
    Json(task_ids)
}

async fn cancel_task(
    State(state): State<AppState>,
    axum::extract::Path(task_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    if let Some(task) = state.scheduler.get_task(&task_id).await {
        task.cancel().await;
        Ok(Json(serde_json::json!({"status": "cancelled"})))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}
*/
