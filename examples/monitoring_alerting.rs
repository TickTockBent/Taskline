//! Real-world example: Monitoring and alerting system
//!
//! This example demonstrates:
//! - System resource monitoring (CPU, memory, disk)
//! - Threshold-based alerting
//! - Log file monitoring and analysis
//! - Service availability checks
//! - Alert rate limiting and deduplication

use taskline::{Scheduler, Task, SchedulerEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Clone)]
struct MonitoringSystem {
    scheduler: Arc<Scheduler>,
    metrics: Arc<RwLock<SystemMetrics>>,
    alerts: Arc<RwLock<AlertManager>>,
}

#[derive(Default)]
struct SystemMetrics {
    cpu_usage: f64,
    memory_usage: f64,
    disk_usage: f64,
    network_latency_ms: u64,
    error_rate: f64,
    service_status: HashMap<String, ServiceStatus>,
}

#[derive(Clone)]
struct ServiceStatus {
    name: String,
    is_healthy: bool,
    last_check: DateTime<Utc>,
    response_time_ms: u64,
}

struct AlertManager {
    active_alerts: Vec<Alert>,
    alert_history: Vec<Alert>,
    alert_cooldowns: HashMap<String, DateTime<Utc>>,
}

#[derive(Clone)]
struct Alert {
    id: String,
    severity: AlertSeverity,
    title: String,
    message: String,
    timestamp: DateTime<Utc>,
    metric_value: f64,
    threshold: f64,
}

#[derive(Clone, PartialEq)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    fn emoji(&self) -> &str {
        match self {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Critical => "🚨",
        }
    }
}

impl AlertManager {
    fn new() -> Self {
        Self {
            active_alerts: Vec::new(),
            alert_history: Vec::new(),
            alert_cooldowns: HashMap::new(),
        }
    }

    fn should_send_alert(&mut self, alert_type: &str) -> bool {
        // Rate limiting: Don't send same alert within 5 minutes
        if let Some(last_sent) = self.alert_cooldowns.get(alert_type) {
            let cooldown = Duration::from_secs(300); // 5 minutes
            if Utc::now().signed_duration_since(*last_sent).to_std().unwrap_or_default() < cooldown {
                return false;
            }
        }
        true
    }

    fn send_alert(&mut self, alert: Alert) {
        println!("{} Alert: {}", alert.severity.emoji(), alert.title);
        println!("   Message: {}", alert.message);
        println!("   Value: {:.2}, Threshold: {:.2}", alert.metric_value, alert.threshold);
        println!("   Time: {}\n", alert.timestamp.format("%Y-%m-%d %H:%M:%S"));

        // Record cooldown
        let alert_key = format!("{}_{}", alert.title, alert.severity.emoji());
        self.alert_cooldowns.insert(alert_key, alert.timestamp);

        self.active_alerts.push(alert.clone());
        self.alert_history.push(alert);
    }

    fn resolve_alert(&mut self, alert_id: &str) {
        self.active_alerts.retain(|a| a.id != alert_id);
        println!("✅ Alert {} resolved", alert_id);
    }

    fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.clone()
    }
}

impl SystemMetrics {
    async fn collect_cpu_usage(&mut self) -> f64 {
        // Simulate CPU usage collection
        self.cpu_usage = 20.0 + (rand::random::<f64>() * 60.0);
        self.cpu_usage
    }

    async fn collect_memory_usage(&mut self) -> f64 {
        // Simulate memory usage collection
        self.memory_usage = 40.0 + (rand::random::<f64>() * 40.0);
        self.memory_usage
    }

    async fn collect_disk_usage(&mut self) -> f64 {
        // Simulate disk usage collection
        self.disk_usage = 50.0 + (rand::random::<f64>() * 30.0);
        self.disk_usage
    }

    async fn check_service(&mut self, service_name: &str, endpoint: &str) -> ServiceStatus {
        // Simulate service health check
        tokio::time::sleep(Duration::from_millis(10)).await;

        let is_healthy = rand::random::<f64>() > 0.1; // 90% success rate
        let response_time = if is_healthy {
            rand::random::<u64>() % 200
        } else {
            rand::random::<u64>() % 5000
        };

        let status = ServiceStatus {
            name: service_name.to_string(),
            is_healthy,
            last_check: Utc::now(),
            response_time_ms: response_time,
        };

        self.service_status.insert(service_name.to_string(), status.clone());
        status
    }

    async fn analyze_error_logs(&mut self) -> f64 {
        // Simulate error log analysis
        self.error_rate = rand::random::<f64>() * 5.0; // 0-5% error rate
        self.error_rate
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔍 Starting Monitoring & Alerting System\n");

    let scheduler = Arc::new(Scheduler::new());
    let metrics = Arc::new(RwLock::new(SystemMetrics::default()));
    let alerts = Arc::new(RwLock::new(AlertManager::new()));

    let monitoring = MonitoringSystem {
        scheduler: scheduler.clone(),
        metrics: metrics.clone(),
        alerts: alerts.clone(),
    };

    // Subscribe to scheduler events
    let mut event_receiver = scheduler.event_bus().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskFailed { task_name, error, .. } => {
                    eprintln!("⚠️  Monitoring task '{}' failed: {}", task_name, error);
                }
                _ => {}
            }
        }
    });

    setup_monitoring_tasks(&monitoring).await?;

    scheduler.start().await?;

    println!("📊 Active monitoring tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            println!("  • {}", task.name());
        }
    }
    println!();

    // Run for demo duration
    println!("▶️  Monitoring system running (30 seconds)...\n");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Show summary
    println!("\n📈 Monitoring Summary:");
    let final_metrics = metrics.read().await;
    println!("  CPU Usage: {:.2}%", final_metrics.cpu_usage);
    println!("  Memory Usage: {:.2}%", final_metrics.memory_usage);
    println!("  Disk Usage: {:.2}%", final_metrics.disk_usage);
    println!("  Error Rate: {:.2}%", final_metrics.error_rate);

    let alert_mgr = alerts.read().await;
    println!("\n🔔 Active Alerts: {}", alert_mgr.get_active_alerts().len());
    println!("📜 Total Alerts Sent: {}", alert_mgr.alert_history.len());

    scheduler.stop().await?;
    println!("\n✅ Monitoring system shutdown complete");

    Ok(())
}

async fn setup_monitoring_tasks(monitoring: &MonitoringSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: CPU monitoring every 10 seconds
    let metrics_clone = monitoring.metrics.clone();
    let alerts_clone = monitoring.alerts.clone();
    let cpu_monitor = Task::new(move || {
        let metrics = metrics_clone.clone();
        let alerts = alerts_clone.clone();
        async move {
            let mut m = metrics.write().await;
            let cpu = m.collect_cpu_usage().await;

            println!("🖥️  CPU: {:.1}%", cpu);

            // Check thresholds
            if cpu > 80.0 {
                let mut alert_mgr = alerts.write().await;
                if alert_mgr.should_send_alert("cpu_critical") {
                    alert_mgr.send_alert(Alert {
                        id: format!("cpu_{}", Utc::now().timestamp()),
                        severity: AlertSeverity::Critical,
                        title: "High CPU Usage".to_string(),
                        message: "CPU usage exceeds 80%".to_string(),
                        timestamp: Utc::now(),
                        metric_value: cpu,
                        threshold: 80.0,
                    });
                }
            } else if cpu > 60.0 {
                let mut alert_mgr = alerts.write().await;
                if alert_mgr.should_send_alert("cpu_warning") {
                    alert_mgr.send_alert(Alert {
                        id: format!("cpu_{}", Utc::now().timestamp()),
                        severity: AlertSeverity::Warning,
                        title: "Elevated CPU Usage".to_string(),
                        message: "CPU usage exceeds 60%".to_string(),
                        timestamp: Utc::now(),
                        metric_value: cpu,
                        threshold: 60.0,
                    });
                }
            }

            Ok(())
        }
    })
    .with_name("CPU Monitor")
    .with_interval(Duration::from_secs(10))
    .with_tags(&["monitoring", "cpu", "metrics"]);

    monitoring.scheduler.add_task(cpu_monitor)?;

    // Task 2: Memory monitoring every 10 seconds
    let metrics_clone = monitoring.metrics.clone();
    let alerts_clone = monitoring.alerts.clone();
    let memory_monitor = Task::new(move || {
        let metrics = metrics_clone.clone();
        let alerts = alerts_clone.clone();
        async move {
            let mut m = metrics.write().await;
            let memory = m.collect_memory_usage().await;

            println!("💾 Memory: {:.1}%", memory);

            if memory > 85.0 {
                let mut alert_mgr = alerts.write().await;
                if alert_mgr.should_send_alert("memory_critical") {
                    alert_mgr.send_alert(Alert {
                        id: format!("mem_{}", Utc::now().timestamp()),
                        severity: AlertSeverity::Critical,
                        title: "High Memory Usage".to_string(),
                        message: "Memory usage exceeds 85%".to_string(),
                        timestamp: Utc::now(),
                        metric_value: memory,
                        threshold: 85.0,
                    });
                }
            }

            Ok(())
        }
    })
    .with_name("Memory Monitor")
    .with_interval(Duration::from_secs(10))
    .with_tags(&["monitoring", "memory", "metrics"]);

    monitoring.scheduler.add_task(memory_monitor)?;

    // Task 3: Disk space monitoring every 5 minutes
    let metrics_clone = monitoring.metrics.clone();
    let alerts_clone = monitoring.alerts.clone();
    let disk_monitor = Task::new(move || {
        let metrics = metrics_clone.clone();
        let alerts = alerts_clone.clone();
        async move {
            let mut m = metrics.write().await;
            let disk = m.collect_disk_usage().await;

            println!("💿 Disk: {:.1}%", disk);

            if disk > 90.0 {
                let mut alert_mgr = alerts.write().await;
                if alert_mgr.should_send_alert("disk_critical") {
                    alert_mgr.send_alert(Alert {
                        id: format!("disk_{}", Utc::now().timestamp()),
                        severity: AlertSeverity::Critical,
                        title: "Critical Disk Space".to_string(),
                        message: "Disk usage exceeds 90%".to_string(),
                        timestamp: Utc::now(),
                        metric_value: disk,
                        threshold: 90.0,
                    });
                }
            }

            Ok(())
        }
    })
    .with_name("Disk Monitor")
    .with_interval(Duration::from_secs(15))
    .with_tags(&["monitoring", "disk", "storage"]);

    monitoring.scheduler.add_task(disk_monitor)?;

    // Task 4: Service health checks every 30 seconds
    let metrics_clone = monitoring.metrics.clone();
    let alerts_clone = monitoring.alerts.clone();
    let health_check = Task::new(move || {
        let metrics = metrics_clone.clone();
        let alerts = alerts_clone.clone();
        async move {
            let mut m = metrics.write().await;

            // Check multiple services
            let services = vec![
                ("API Gateway", "https://api.example.com/health"),
                ("Database", "postgres://db.example.com:5432"),
                ("Cache", "redis://cache.example.com:6379"),
            ];

            for (name, endpoint) in services {
                let status = m.check_service(name, endpoint).await;

                let icon = if status.is_healthy { "✅" } else { "❌" };
                println!("{} Service '{}': {} ({}ms)",
                    icon, name,
                    if status.is_healthy { "UP" } else { "DOWN" },
                    status.response_time_ms
                );

                if !status.is_healthy {
                    let mut alert_mgr = alerts.write().await;
                    let alert_key = format!("service_{}", name);
                    if alert_mgr.should_send_alert(&alert_key) {
                        alert_mgr.send_alert(Alert {
                            id: format!("svc_{}_{}", name, Utc::now().timestamp()),
                            severity: AlertSeverity::Critical,
                            title: format!("Service '{}' Down", name),
                            message: format!("Service '{}' failed health check", name),
                            timestamp: Utc::now(),
                            metric_value: 0.0,
                            threshold: 1.0,
                        });
                    }
                }
            }

            Ok(())
        }
    })
    .with_name("Service Health Check")
    .with_interval(Duration::from_secs(30))
    .with_tags(&["monitoring", "health", "services"]);

    monitoring.scheduler.add_task(health_check)?;

    // Task 5: Error log analysis every 2 minutes
    let metrics_clone = monitoring.metrics.clone();
    let alerts_clone = monitoring.alerts.clone();
    let log_analyzer = Task::new(move || {
        let metrics = metrics_clone.clone();
        let alerts = alerts_clone.clone();
        async move {
            let mut m = metrics.write().await;
            let error_rate = m.analyze_error_logs().await;

            println!("📋 Error Rate: {:.2}%", error_rate);

            if error_rate > 3.0 {
                let mut alert_mgr = alerts.write().await;
                if alert_mgr.should_send_alert("error_rate_high") {
                    alert_mgr.send_alert(Alert {
                        id: format!("err_{}", Utc::now().timestamp()),
                        severity: AlertSeverity::Warning,
                        title: "High Error Rate".to_string(),
                        message: format!("Error rate at {:.2}%", error_rate),
                        timestamp: Utc::now(),
                        metric_value: error_rate,
                        threshold: 3.0,
                    });
                }
            }

            Ok(())
        }
    })
    .with_name("Log Analyzer")
    .with_interval(Duration::from_secs(12))
    .with_tags(&["monitoring", "logs", "errors"]);

    monitoring.scheduler.add_task(log_analyzer)?;

    Ok(())
}
