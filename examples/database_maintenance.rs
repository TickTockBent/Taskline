#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(clippy::all)]
//! Allow dead code in examples
#![allow(dead_code)]
#![allow(unused_variables)]

//! Real-world example: Database maintenance tasks
//!
//! This example demonstrates:
//! - Scheduled database backups
//! - Index rebuilding and optimization
//! - Data archival and purging
//! - Vacuum operations
//! - Statistics updates

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use cronline::{Scheduler, SchedulerEvent, Task, TaskConfig};
use tokio::sync::RwLock;

#[derive(Clone)]
struct DatabaseManager {
    scheduler: Arc<Scheduler>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
    backup_manager: Arc<RwLock<BackupManager>>,
}

struct ConnectionPool {
    active_connections: usize,
    max_connections: usize,
    total_queries_executed: u64,
}

struct BackupManager {
    backup_directory: String,
    last_backup: Option<DateTime<Utc>>,
    backup_history: Vec<BackupRecord>,
}

#[derive(Clone)]
struct BackupRecord {
    timestamp: DateTime<Utc>,
    size_mb: u64,
    duration_seconds: u64,
    status: BackupStatus,
}

#[derive(Clone)]
enum BackupStatus {
    Success,
    Failed(String),
}

impl ConnectionPool {
    fn new(max_connections: usize) -> Self {
        Self {
            active_connections: 0,
            max_connections,
            total_queries_executed: 0,
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<(), String> {
        // Simulate query execution
        self.total_queries_executed += 1;
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("  🔹 Executed: {}", query);
        Ok(())
    }

    async fn vacuum_database(&mut self) -> Result<(), String> {
        println!("  🧹 Starting VACUUM operation...");
        self.execute_query("VACUUM FULL ANALYZE").await?;
        println!("  ✅ VACUUM completed successfully");
        Ok(())
    }

    async fn rebuild_indexes(&mut self) -> Result<usize, String> {
        println!("  🔧 Rebuilding database indexes...");
        let indexes = vec![
            "idx_users_email",
            "idx_orders_date",
            "idx_products_category",
        ];

        for (i, index) in indexes.iter().enumerate() {
            let query = format!("REINDEX INDEX {}", index);
            self.execute_query(&query).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!("  ✅ Rebuilt {} indexes", indexes.len());
        Ok(indexes.len())
    }

    async fn update_statistics(&mut self) -> Result<(), String> {
        println!("  📊 Updating table statistics...");
        self.execute_query("ANALYZE").await?;
        println!("  ✅ Statistics updated");
        Ok(())
    }

    async fn purge_old_data(&mut self, days: u32) -> Result<usize, String> {
        println!("  🗑️  Purging data older than {} days...", days);

        // Simulate deletion
        let query = format!(
            "DELETE FROM logs WHERE created_at < NOW() - INTERVAL '{} days'",
            days
        );
        self.execute_query(&query).await?;

        let rows_deleted = (rand::random::<u32>() % 10000) as usize;
        println!("  ✅ Purged {} rows", rows_deleted);
        Ok(rows_deleted)
    }

    async fn archive_old_records(&mut self, table: &str, days: u32) -> Result<usize, String> {
        println!(
            "  📦 Archiving {} records older than {} days...",
            table, days
        );

        // Simulate archival
        let insert_query = format!(
            "INSERT INTO {}_archive SELECT * FROM {} WHERE created_at < NOW() - INTERVAL '{} days'",
            table, table, days
        );
        self.execute_query(&insert_query).await?;

        let delete_query = format!(
            "DELETE FROM {} WHERE created_at < NOW() - INTERVAL '{} days'",
            table, days
        );
        self.execute_query(&delete_query).await?;

        let rows_archived = (rand::random::<u32>() % 5000) as usize;
        println!("  ✅ Archived {} rows from {}", rows_archived, table);
        Ok(rows_archived)
    }

    async fn check_table_bloat(&mut self) -> Result<Vec<String>, String> {
        println!("  🔍 Checking for table bloat...");
        self.execute_query("SELECT * FROM pg_stat_user_tables")
            .await?;

        // Simulate finding bloated tables
        let bloated = if rand::random::<f64>() > 0.7 {
            vec!["orders".to_string(), "logs".to_string()]
        } else {
            vec![]
        };

        if !bloated.is_empty() {
            println!(
                "  ⚠️  Found {} bloated tables: {:?}",
                bloated.len(),
                bloated
            );
        } else {
            println!("  ✅ No significant bloat detected");
        }

        Ok(bloated)
    }
}

impl BackupManager {
    fn new(backup_directory: String) -> Self {
        Self {
            backup_directory,
            last_backup: None,
            backup_history: Vec::new(),
        }
    }

    async fn perform_full_backup(&mut self) -> Result<BackupRecord, String> {
        let start = Utc::now();
        println!("💾 Starting full database backup...");
        println!("   Backup location: {}", self.backup_directory);

        // Simulate backup process
        tokio::time::sleep(Duration::from_millis(500)).await;

        let duration = Utc::now().signed_duration_since(start).num_seconds() as u64;

        let record = BackupRecord {
            timestamp: start,
            size_mb: 150 + (rand::random::<u64>() % 50),
            duration_seconds: duration,
            status: BackupStatus::Success,
        };

        self.last_backup = Some(record.timestamp);
        self.backup_history.push(record.clone());

        println!("✅ Backup completed successfully!");
        println!("   Size: {} MB", record.size_mb);
        println!("   Duration: {} seconds", record.duration_seconds);

        Ok(record)
    }

    async fn perform_incremental_backup(&mut self) -> Result<BackupRecord, String> {
        let start = Utc::now();
        println!("💾 Starting incremental backup...");

        // Simulate backup
        tokio::time::sleep(Duration::from_millis(200)).await;

        let duration = Utc::now().signed_duration_since(start).num_seconds() as u64;

        let record = BackupRecord {
            timestamp: start,
            size_mb: 10 + (rand::random::<u64>() % 20),
            duration_seconds: duration,
            status: BackupStatus::Success,
        };

        self.backup_history.push(record.clone());
        println!("✅ Incremental backup completed - {} MB", record.size_mb);

        Ok(record)
    }

    async fn cleanup_old_backups(&mut self, retain_days: u32) -> Result<usize, String> {
        println!("🧹 Cleaning up backups older than {} days...", retain_days);

        // Simulate cleanup
        let cutoff = Utc::now() - chrono::Duration::days(retain_days as i64);
        let before_count = self.backup_history.len();

        self.backup_history
            .retain(|backup| backup.timestamp > cutoff);

        let removed = before_count - self.backup_history.len();
        println!("✅ Removed {} old backup(s)", removed);

        Ok(removed)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init(); // Optional: enable with basic-logging feature

    println!("🗄️  Starting Database Maintenance System\n");

    let scheduler = Arc::new(Scheduler::new());
    let connection_pool = Arc::new(RwLock::new(ConnectionPool::new(100)));
    let backup_manager = Arc::new(RwLock::new(BackupManager::new(
        "/backups/database".to_string(),
    )));

    let db_manager = DatabaseManager {
        scheduler: scheduler.clone(),
        connection_pool: connection_pool.clone(),
        backup_manager: backup_manager.clone(),
    };

    // Subscribe to events
    let mut event_receiver = scheduler.event_bus().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskCompleted {
                    task_name,
                    duration_ms,
                    ..
                } => {
                    println!("✅ '{}' completed in {}ms\n", task_name, duration_ms);
                }
                SchedulerEvent::TaskFailed {
                    task_name, error, ..
                } => {
                    eprintln!("❌ '{}' failed: {}\n", task_name, error);
                }
                _ => {}
            }
        }
    });

    setup_maintenance_tasks(&db_manager).await?;

    scheduler.start().await?;

    println!("📋 Scheduled maintenance tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            let stats = task.stats().await;
            println!("  • {} (Tags: {:?})", task.name(), task.tags());
            if let Some(next) = stats.next_execution {
                println!("    Next run: {}", next.format("%Y-%m-%d %H:%M:%S"));
            }
        }
    }
    println!();

    // Run demo
    println!("▶️  Running maintenance tasks (30 seconds)...\n");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Summary
    let pool = connection_pool.read().await;
    let backups = backup_manager.read().await;

    println!("\n📊 Maintenance Summary:");
    println!("  Total queries executed: {}", pool.total_queries_executed);
    println!("  Backups performed: {}", backups.backup_history.len());
    if let Some(last) = backups.last_backup {
        println!("  Last backup: {}", last.format("%Y-%m-%d %H:%M:%S"));
    }

    scheduler.stop().await?;
    println!("\n✅ Database maintenance system shutdown complete");

    Ok(())
}

async fn setup_maintenance_tasks(db: &DatabaseManager) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: Full backup - Daily at 2 AM
    let backup_mgr = db.backup_manager.clone();
    let full_backup_task = Task::new(move || {
        let mgr = backup_mgr.clone();
        async move {
            let mut backup = mgr.write().await;
            match backup.perform_full_backup().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_schedule("0 2 * * *")?
    .with_tags(&["backup", "database", "critical"])
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(3600)), // 1 hour
        max_retries: 2,
        retry_delay: Duration::from_secs(300),
        fail_scheduler_on_error: false,
    });

    db.scheduler.add_task(full_backup_task).await?;

    // Task 2: Incremental backup - Every 6 hours
    let backup_mgr = db.backup_manager.clone();
    let incremental_backup_task = Task::new(move || {
        let mgr = backup_mgr.clone();
        async move {
            let mut backup = mgr.write().await;
            match backup.perform_incremental_backup().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_interval(Duration::from_secs(15)) // Demo: 15 seconds
    .with_tags(&["backup", "database"]);

    db.scheduler.add_task(incremental_backup_task).await?;

    // Task 3: Vacuum database - Weekly on Sunday at 3 AM
    let pool = db.connection_pool.clone();
    let vacuum_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.vacuum_database().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_schedule("0 3 * * 0")?
    .with_tags(&["maintenance", "vacuum", "weekly"]);

    db.scheduler.add_task(vacuum_task).await?;

    // Task 4: Rebuild indexes - Monthly on first day at 4 AM
    let pool = db.connection_pool.clone();
    let index_rebuild_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.rebuild_indexes().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_schedule("0 4 1 * *")?
    .with_tags(&["maintenance", "indexes", "monthly"]);

    db.scheduler.add_task(index_rebuild_task).await?;

    // Task 5: Update statistics - Daily at 1 AM
    let pool = db.connection_pool.clone();
    let stats_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.update_statistics().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_interval(Duration::from_secs(20)) // Demo: 20 seconds
    .with_tags(&["maintenance", "statistics"]);

    db.scheduler.add_task(stats_task).await?;

    // Task 6: Purge old logs - Daily at 5 AM
    let pool = db.connection_pool.clone();
    let purge_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.purge_old_data(90).await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_schedule("0 5 * * *")?
    .with_tags(&["cleanup", "logs", "daily"]);

    db.scheduler.add_task(purge_task).await?;

    // Task 7: Archive old orders - Weekly
    let pool = db.connection_pool.clone();
    let archive_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.archive_old_records("orders", 365).await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_interval(Duration::from_secs(25)) // Demo: 25 seconds
    .with_tags(&["archival", "orders", "weekly"]);

    db.scheduler.add_task(archive_task).await?;

    // Task 8: Check table bloat - Daily
    let pool = db.connection_pool.clone();
    let bloat_check_task = Task::new(move || {
        let pool = pool.clone();
        async move {
            let mut db = pool.write().await;
            match db.check_table_bloat().await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_interval(Duration::from_secs(18)) // Demo: 18 seconds
    .with_tags(&["monitoring", "bloat", "health"]);

    db.scheduler.add_task(bloat_check_task).await?;

    // Task 9: Cleanup old backups - Weekly
    let backup_mgr = db.backup_manager.clone();
    let backup_cleanup_task = Task::new(move || {
        let mgr = backup_mgr.clone();
        async move {
            let mut backup = mgr.write().await;
            match backup.cleanup_old_backups(30).await {
                Ok(_) => Ok(()),
                Err(e) => Err(cronline::CronlineError::TaskExecutionError(e)),
            }
        }
    })
    .with_schedule("0 6 * * 0")?
    .with_tags(&["cleanup", "backup", "weekly"]);

    db.scheduler.add_task(backup_cleanup_task).await?;

    Ok(())
}
