//! Real-world example: Email and notification sending system
//!
//! This example demonstrates:
//! - Scheduled email campaigns
//! - Reminder notifications
//! - Digest emails (daily/weekly summaries)
//! - Event-triggered notifications
//! - Multi-channel notifications (email, SMS, push)

use taskline::{Scheduler, Task, TaskConfig, SchedulerEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Weekday};
use std::collections::VecDeque;

#[derive(Clone)]
struct NotificationSystem {
    scheduler: Arc<Scheduler>,
    email_service: Arc<RwLock<EmailService>>,
    notification_queue: Arc<RwLock<NotificationQueue>>,
}

struct EmailService {
    sent_emails: u64,
    failed_emails: u64,
    email_history: VecDeque<EmailRecord>,
}

#[derive(Clone)]
struct EmailRecord {
    to: String,
    subject: String,
    sent_at: DateTime<Utc>,
    delivery_status: DeliveryStatus,
}

#[derive(Clone)]
enum DeliveryStatus {
    Sent,
    Failed(String),
    Pending,
}

struct NotificationQueue {
    pending: Vec<Notification>,
    sent: Vec<Notification>,
}

#[derive(Clone)]
struct Notification {
    id: String,
    recipient: String,
    channel: NotificationChannel,
    message: String,
    priority: Priority,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
enum NotificationChannel {
    Email,
    Sms,
    PushNotification,
    Webhook,
}

#[derive(Clone, PartialEq, PartialOrd)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl EmailService {
    fn new() -> Self {
        Self {
            sent_emails: 0,
            failed_emails: 0,
            email_history: VecDeque::new(),
        }
    }

    async fn send_email(&mut self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        // Simulate sending email
        tokio::time::sleep(Duration::from_millis(100)).await;

        let success = rand::random::<f64>() > 0.05; // 95% success rate

        let record = EmailRecord {
            to: to.to_string(),
            subject: subject.to_string(),
            sent_at: Utc::now(),
            delivery_status: if success {
                DeliveryStatus::Sent
            } else {
                DeliveryStatus::Failed("SMTP error".to_string())
            },
        };

        if success {
            self.sent_emails += 1;
            println!("✉️  Sent: {} -> '{}'", to, subject);
        } else {
            self.failed_emails += 1;
            println!("❌ Failed: {} -> '{}'", to, subject);
        }

        self.email_history.push_back(record);

        // Keep only last 100 emails
        if self.email_history.len() > 100 {
            self.email_history.pop_front();
        }

        if success {
            Ok(())
        } else {
            Err("Failed to send email".to_string())
        }
    }

    async fn send_bulk_email(&mut self, recipients: Vec<&str>, subject: &str, body: &str) -> Result<usize, String> {
        println!("📧 Sending bulk email to {} recipient(s)...", recipients.len());

        let mut sent_count = 0;
        for recipient in recipients {
            if self.send_email(recipient, subject, body).await.is_ok() {
                sent_count += 1;
            }
            // Rate limiting: Small delay between emails
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        println!("✅ Bulk send complete: {}/{} successful", sent_count, recipients.len());
        Ok(sent_count)
    }

    async fn send_digest(&mut self, recipient: &str, items: Vec<String>) -> Result<(), String> {
        let subject = "Daily Digest - Summary of Activities";
        let body = format!("Here are your updates:\n{}", items.join("\n"));

        self.send_email(recipient, subject, &body).await
    }
}

impl NotificationQueue {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
            sent: Vec::new(),
        }
    }

    fn add(&mut self, notification: Notification) {
        self.pending.push(notification);
        self.pending.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    }

    async fn process_queue(&mut self, max_batch: usize) -> Result<usize, String> {
        let batch: Vec<_> = self.pending.drain(..max_batch.min(self.pending.len())).collect();
        let count = batch.len();

        if count == 0 {
            return Ok(0);
        }

        println!("📤 Processing {} notification(s)...", count);

        for notification in batch {
            // Simulate sending notification
            tokio::time::sleep(Duration::from_millis(50)).await;

            match notification.channel {
                NotificationChannel::Email => {
                    println!("  ✉️  Email to {}: {}", notification.recipient, notification.message);
                }
                NotificationChannel::Sms => {
                    println!("  📱 SMS to {}: {}", notification.recipient, notification.message);
                }
                NotificationChannel::PushNotification => {
                    println!("  🔔 Push to {}: {}", notification.recipient, notification.message);
                }
                NotificationChannel::Webhook => {
                    println!("  🔗 Webhook to {}: {}", notification.recipient, notification.message);
                }
            }

            self.sent.push(notification);
        }

        Ok(count)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("📬 Starting Notification System\n");

    let scheduler = Arc::new(Scheduler::new());
    let email_service = Arc::new(RwLock::new(EmailService::new()));
    let notification_queue = Arc::new(RwLock::new(NotificationQueue::new()));

    let system = NotificationSystem {
        scheduler: scheduler.clone(),
        email_service: email_service.clone(),
        notification_queue: notification_queue.clone(),
    };

    // Subscribe to events
    let mut event_receiver = scheduler.event_bus().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskFailed { task_name, error, .. } => {
                    eprintln!("⚠️  Notification task '{}' failed: {}", task_name, error);
                }
                _ => {}
            }
        }
    });

    // Add some test notifications to queue
    {
        let mut queue = notification_queue.write().await;
        queue.add(Notification {
            id: "1".to_string(),
            recipient: "user@example.com".to_string(),
            channel: NotificationChannel::Email,
            message: "Your order has shipped!".to_string(),
            priority: Priority::High,
            created_at: Utc::now(),
        });
        queue.add(Notification {
            id: "2".to_string(),
            recipient: "+1234567890".to_string(),
            channel: NotificationChannel::Sms,
            message: "Your verification code: 123456".to_string(),
            priority: Priority::Critical,
            created_at: Utc::now(),
        });
    }

    setup_notification_tasks(&system).await?;

    scheduler.start().await?;

    println!("📋 Scheduled notification tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            println!("  • {} (Tags: {:?})", task.name(), task.tags());
        }
    }
    println!("\n▶️  Running notification system (30 seconds)...\n");

    tokio::time::sleep(Duration::from_secs(30)).await;

    // Summary
    println!("\n📊 Notification Summary:");
    let email_svc = email_service.read().await;
    let queue = notification_queue.read().await;

    println!("  Emails sent: {}", email_svc.sent_emails);
    println!("  Emails failed: {}", email_svc.failed_emails);
    println!("  Notifications sent: {}", queue.sent.len());
    println!("  Notifications pending: {}", queue.pending.len());

    scheduler.stop().await?;
    println!("\n✅ Notification system shutdown complete");

    Ok(())
}

async fn setup_notification_tasks(system: &NotificationSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: Daily digest - Every day at 8 AM
    let email_service = system.email_service.clone();
    let daily_digest = Task::new(move || {
        let service = email_service.clone();
        async move {
            let mut email = service.write().await;

            let digest_items = vec![
                "• 5 new messages".to_string(),
                "• 3 tasks completed".to_string(),
                "• 2 upcoming events".to_string(),
            ];

            email.send_digest("user@example.com", digest_items).await
                .map_err(|e| taskline::TasklineError::TaskExecutionError(e))
        }
    })
    .with_schedule("0 8 * * *")?
    .with_tags(&["email", "digest", "daily"]);

    system.scheduler.add_task(daily_digest)?;

    // Task 2: Weekly report - Every Monday at 9 AM
    let email_service = system.email_service.clone();
    let weekly_report = Task::new(move || {
        let service = email_service.clone();
        async move {
            let mut email = service.write().await;

            println!("📊 Generating weekly report...");
            let subject = "Weekly Activity Report";
            let body = "Your weekly summary:\n\n- Total activities: 42\n- Goals completed: 8\n- Time saved: 3 hours";

            email.send_email("manager@example.com", subject, body).await
                .map_err(|e| taskline::TasklineError::TaskExecutionError(e))
        }
    })
    .with_schedule("0 9 * * 1")?  // Monday at 9 AM
    .with_tags(&["email", "report", "weekly"]);

    system.scheduler.add_task(weekly_report)?;

    // Task 3: Reminder notifications - Every hour
    let email_service = system.email_service.clone();
    let reminders = Task::new(move || {
        let service = email_service.clone();
        async move {
            let mut email = service.write().await;

            // Simulate checking for upcoming events
            let has_reminder = rand::random::<f64>() > 0.6;

            if has_reminder {
                email.send_email(
                    "user@example.com",
                    "Reminder: Upcoming Meeting",
                    "You have a meeting in 30 minutes"
                ).await.map_err(|e| taskline::TasklineError::TaskExecutionError(e))?;
            }

            Ok(())
        }
    })
    .with_interval(Duration::from_secs(12)) // Demo: 12 seconds
    .with_tags(&["reminder", "notification"]);

    system.scheduler.add_task(reminders)?;

    // Task 4: Process notification queue - Every 30 seconds
    let queue = system.notification_queue.clone();
    let queue_processor = Task::new(move || {
        let q = queue.clone();
        async move {
            let mut queue = q.write().await;
            queue.process_queue(5).await
                .map_err(|e| taskline::TasklineError::TaskExecutionError(e))?;
            Ok(())
        }
    })
    .with_name("Notification Queue Processor")
    .with_interval(Duration::from_secs(8)) // Demo: 8 seconds
    .with_tags(&["queue", "notification", "processing"]);

    system.scheduler.add_task(queue_processor)?;

    // Task 5: Marketing campaign - Weekly on Thursday at 10 AM
    let email_service = system.email_service.clone();
    let marketing_campaign = Task::new(move || {
        let service = email_service.clone();
        async move {
            let mut email = service.write().await;

            let recipients = vec![
                "customer1@example.com",
                "customer2@example.com",
                "customer3@example.com",
            ];

            email.send_bulk_email(
                recipients,
                "Special Offer: 20% Off This Week!",
                "Don't miss our exclusive weekly deals..."
            ).await.map_err(|e| taskline::TasklineError::TaskExecutionError(e))?;

            Ok(())
        }
    })
    .with_schedule("0 10 * * 4")?  // Thursday at 10 AM
    .with_tags(&["marketing", "campaign", "bulk"])
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(300)),
        max_retries: 2,
        retry_delay: Duration::from_secs(60),
        fail_scheduler_on_error: false,
    });

    system.scheduler.add_task(marketing_campaign)?;

    // Task 6: Abandoned cart reminders - Every 6 hours
    let email_service = system.email_service.clone();
    let cart_reminders = Task::new(move || {
        let service = email_service.clone();
        async move {
            let mut email = service.write().await;

            // Simulate checking for abandoned carts
            let has_abandoned = rand::random::<f64>() > 0.5;

            if has_abandoned {
                email.send_email(
                    "shopper@example.com",
                    "Don't Forget Your Cart!",
                    "You left items in your cart. Complete your purchase now!"
                ).await.map_err(|e| taskline::TasklineError::TaskExecutionError(e))?;
            }

            Ok(())
        }
    })
    .with_interval(Duration::from_secs(15)) // Demo: 15 seconds
    .with_tags(&["ecommerce", "cart", "reminder"]);

    system.scheduler.add_task(cart_reminders)?;

    // Task 7: System alerts - Immediate processing
    let queue = system.notification_queue.clone();
    let alert_processor = Task::new(move || {
        let q = queue.clone();
        async move {
            let mut queue = q.write().await;

            // Add some test alerts
            if rand::random::<f64>() > 0.7 {
                queue.add(Notification {
                    id: format!("alert_{}", Utc::now().timestamp()),
                    recipient: "admin@example.com".to_string(),
                    channel: NotificationChannel::Email,
                    message: "System alert: High CPU usage detected".to_string(),
                    priority: Priority::Critical,
                    created_at: Utc::now(),
                });
            }

            Ok(())
        }
    })
    .with_name("Alert Generator")
    .with_interval(Duration::from_secs(10))
    .with_tags(&["alerts", "monitoring"]);

    system.scheduler.add_task(alert_processor)?;

    Ok(())
}
