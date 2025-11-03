# Cronline Real-World Examples

This document provides an overview of the comprehensive real-world examples included with Cronline. Each example demonstrates practical applications and best practices for using Cronline in production scenarios.

## Running the Examples

All examples can be run using:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example web_integration_axum
cargo run --example monitoring_alerting
```

---

## 1. Web Framework Integration (Axum)

**File:** `examples/web_integration_axum.rs`

**What it demonstrates:**
- Integrating Cronline with the Axum web framework
- Running a scheduler alongside a web server
- Health check tasks running in the background
- Scheduled database maintenance
- Metrics collection for web applications
- Event-driven architecture with the event bus

**Use cases:**
- Web services that need background jobs
- API servers with periodic tasks
- Microservices with scheduled maintenance
- SaaS applications with recurring operations

**Key features shown:**
```rust
// Health check every 30 seconds
Task::new(|| async { /* health check */ })
    .with_interval(Duration::from_secs(30))
    .with_tags(&["monitoring", "health", "critical"])

// Database cleanup every 5 minutes
Task::new(|| async { /* cleanup */ })
    .with_schedule("*/5 * * * *")?
    .with_tags(&["database", "maintenance"])

// Subscribe to events
let mut events = scheduler.event_bus().subscribe();
```

**Topics covered:**
- Application state management
- Task configuration with timeouts and retries
- Event subscription and handling
- Tag-based task organization
- Integration patterns with web frameworks

---

## 2. Monitoring & Alerting System

**File:** `examples/monitoring_alerting.rs`

**What it demonstrates:**
- System resource monitoring (CPU, memory, disk)
- Threshold-based alerting with different severity levels
- Service health checks
- Alert rate limiting and deduplication
- Multi-metric monitoring

**Use cases:**
- Infrastructure monitoring
- Application performance monitoring (APM)
- Service level monitoring
- Automated incident detection
- Proactive alerting systems

**Key features shown:**
```rust
// CPU monitoring with threshold alerts
if cpu > 80.0 {
    alert_mgr.send_alert(Alert {
        severity: AlertSeverity::Critical,
        title: "High CPU Usage".to_string(),
        // ...
    });
}

// Service health checks
Task::new(|| async {
    check_multiple_services().await
})
.with_interval(Duration::from_secs(30))
.with_tags(&["monitoring", "health", "services"])
```

**Topics covered:**
- Real-time metrics collection
- Alert management and prioritization
- Rate limiting for notifications
- Multi-service health checking
- Historical tracking and analysis

---

## 3. Database Maintenance

**File:** `examples/database_maintenance.rs`

**What it demonstrates:**
- Scheduled database backups (full and incremental)
- Index rebuilding and optimization
- Data archival and purging strategies
- VACUUM operations
- Statistics updates
- Backup retention policies

**Use cases:**
- PostgreSQL/MySQL maintenance
- Automated backup systems
- Data lifecycle management
- Database performance optimization
- Compliance-driven data retention

**Key features shown:**
```rust
// Full backup daily at 2 AM
Task::new(|| async { perform_full_backup().await })
    .with_schedule("0 2 * * *")?
    .with_tags(&["backup", "database", "critical"])
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(3600)),
        max_retries: 2,
        // ...
    })

// Purge old logs daily
Task::new(|| async { purge_old_data(90).await })
    .with_schedule("0 5 * * *")?
```

**Topics covered:**
- Backup strategies (full vs incremental)
- Data archival patterns
- Database optimization techniques
- Cleanup and maintenance scheduling
- Error handling and retries for critical tasks

---

## 4. API Polling & Web Scraping

**File:** `examples/api_polling_scraping.rs`

**What it demonstrates:**
- Periodic API polling for data updates
- Rate-limited web scraping
- Change detection and alerting
- External service integration
- RSS feed aggregation
- Product price tracking

**Use cases:**
- Price comparison services
- Stock market data aggregation
- Weather data collection
- News feed aggregation
- Third-party API integration
- Competitive intelligence gathering

**Key features shown:**
```rust
// Stock price monitoring with change detection
Task::new(|| async {
    let new_price = fetch_stock_price("AAPL").await?;
    if price_changed_significantly(new_price) {
        send_alert().await;
    }
    Ok(())
})
.with_interval(Duration::from_secs(60))
.with_tags(&["api", "stocks", "finance"])

// Rate-limited scraping
if rate_limiter.can_make_request("api").await {
    fetch_data().await;
}
```

**Topics covered:**
- API rate limiting strategies
- Change detection algorithms
- Data deduplication
- Multi-source data aggregation
- Respectful web scraping practices

---

## 5. Notification System

**File:** `examples/notification_system.rs`

**What it demonstrates:**
- Scheduled email campaigns
- Reminder notifications
- Daily/weekly digest emails
- Event-triggered notifications
- Multi-channel notifications (Email, SMS, Push, Webhook)
- Notification queue processing
- Priority-based delivery

**Use cases:**
- Email marketing platforms
- Transactional email services
- User notification systems
- Automated reminder services
- Multi-channel messaging platforms

**Key features shown:**
```rust
// Daily digest at 8 AM
Task::new(|| async {
    send_digest(user, daily_summary).await
})
.with_schedule("0 8 * * *")?
.with_tags(&["email", "digest", "daily"])

// Process notification queue with priority
Task::new(|| async {
    queue.process_queue(max_batch).await
})
.with_interval(Duration::from_secs(30))
.with_tags(&["queue", "notification", "processing"])

// Weekly marketing campaign
Task::new(|| async {
    send_bulk_email(recipients, campaign).await
})
.with_schedule("0 10 * * 4")?  // Thursday 10 AM
```

**Topics covered:**
- Email delivery patterns
- Queue management
- Priority-based processing
- Bulk sending strategies
- Multi-channel notification routing
- Campaign scheduling

---

## 6. Cache Warming & Management

**File:** `examples/cache_warming.rs`

**What it demonstrates:**
- Proactive cache warming before traffic spikes
- Multi-tier cache management (L1, L2, CDN)
- Cache invalidation strategies
- Cache hit rate monitoring
- Predictive cache preloading
- Automated cleanup of expired entries

**Use cases:**
- E-commerce platforms
- Content delivery networks
- High-traffic web applications
- API caching layers
- Search result caching

**Key features shown:**
```rust
// Morning cache warmup before traffic spike
Task::new(|| async {
    warm_popular_items().await
})
.with_schedule("0 7 * * *")?
.with_tags(&["cache", "warmup", "morning"])

// Pre-sale cache warmup
Task::new(|| async {
    warm_sale_items().await
})
.with_schedule("0 23 * * 5")?  // Friday 11 PM

// Monitor cache hit rates
Task::new(|| async {
    let hit_rate = calculate_hit_rate().await;
    if hit_rate < threshold {
        alert_low_hit_rate().await;
    }
    Ok(())
})
.with_interval(Duration::from_secs(300))
```

**Topics covered:**
- Multi-tier cache architecture
- Predictive warming strategies
- Cache invalidation patterns
- Performance monitoring
- Memory management
- Traffic pattern optimization

---

## Common Patterns Across Examples

### 1. Event Bus Usage
All examples demonstrate subscribing to scheduler events for monitoring and debugging:

```rust
let mut events = scheduler.event_bus().subscribe();
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            SchedulerEvent::TaskCompleted { .. } => { /* log */ },
            SchedulerEvent::TaskFailed { .. } => { /* alert */ },
            _ => {}
        }
    }
});
```

### 2. Tag-based Organization
Tasks are organized using tags for filtering and management:

```rust
// Add tasks with tags
task.with_tags(&["backup", "critical", "database"])

// Filter tasks by tags
let critical_tasks = scheduler.tasks_with_tag("critical").await;
let backup_tasks = scheduler.tasks_with_all_tags(&["backup", "database"]).await;
```

### 3. Error Handling & Retries
Production-ready error handling with configurable retries:

```rust
.with_config(TaskConfig {
    timeout: Some(Duration::from_secs(60)),
    max_retries: 3,
    retry_delay: Duration::from_secs(5),
    fail_scheduler_on_error: false,
})
```

### 4. Interval vs Cron Scheduling
Examples show when to use each approach:

```rust
// Use intervals for simple periodic tasks
.with_interval(Duration::from_secs(300))

// Use cron for specific times
.with_schedule("0 2 * * *")?  // 2 AM daily
.with_schedule("0 9 * * 1")?  // Monday 9 AM
```

---

## Best Practices Demonstrated

1. **Separation of Concerns**: Each task has a single, well-defined responsibility
2. **Resource Management**: Proper cleanup and resource pooling
3. **Graceful Degradation**: Tasks fail gracefully without crashing the scheduler
4. **Observability**: Comprehensive logging and event emission
5. **Configuration**: Externalized configuration for timeouts and retries
6. **Testing**: Examples include simulation of real-world scenarios
7. **Documentation**: Clear comments explaining the why, not just the what

---

## Performance Considerations

Examples demonstrate:
- Efficient batch processing
- Rate limiting to prevent API abuse
- Connection pooling for databases
- Lazy loading and caching strategies
- Memory-conscious data structures
- Async I/O for concurrent operations

---

## Security Considerations

Examples show:
- Safe handling of credentials (not hardcoded)
- Rate limiting to prevent abuse
- Input validation patterns
- Safe error message handling (no sensitive data leakage)

---

## Next Steps

After reviewing these examples:

1. **Start Simple**: Begin with `basic_scheduler.rs` and `enhanced_features.rs`
2. **Pick Your Use Case**: Choose the example closest to your needs
3. **Customize**: Adapt the patterns to your specific requirements
4. **Test**: Use the event bus to monitor task execution
5. **Deploy**: Follow production best practices from the examples

## Additional Resources

- [Main README](README.md) - Getting started guide
- [API Documentation](https://docs.rs/cronline) - Complete API reference
- [GitHub Issues](https://github.com/TickTockBent/Cronline/issues) - Report bugs or request features

---

**Note**: All examples are self-contained and can run independently. They use simulated operations (no real databases, APIs, etc.) to demonstrate concepts without external dependencies.
