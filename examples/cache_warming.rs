#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(clippy::all)]
//! Allow dead code in examples
#![allow(dead_code)]
#![allow(unused_variables)]

//! Real-world example: Cache warming and management
//!
//! This example demonstrates:
//! - Proactive cache warming before traffic spikes
//! - Cache invalidation strategies
//! - Multi-tier cache management
//! - Cache hit rate monitoring
//! - Predictive cache preloading

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use cronline::{Scheduler, SchedulerEvent, Task};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheSystem {
    scheduler: Arc<Scheduler>,
    l1_cache: Arc<RwLock<Cache>>,  // Fast memory cache
    l2_cache: Arc<RwLock<Cache>>,  // Slower distributed cache
    cdn_cache: Arc<RwLock<Cache>>, // CDN edge cache
    stats: Arc<RwLock<CacheStats>>,
}

struct Cache {
    name: String,
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl_seconds: u64,
}

#[derive(Clone)]
struct CacheEntry {
    key: String,
    value: String,
    size_bytes: usize,
    created_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    access_count: u64,
}

#[derive(Default)]
struct CacheStats {
    l1_hits: u64,
    l1_misses: u64,
    l2_hits: u64,
    l2_misses: u64,
    cdn_hits: u64,
    cdn_misses: u64,
    total_evictions: u64,
    total_warmups: u64,
}

impl Cache {
    fn new(name: &str, max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            name: name.to_string(),
            entries: HashMap::new(),
            max_size,
            ttl_seconds,
        }
    }

    async fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.entries.get_mut(key) {
            // Check if expired
            let age = Utc::now()
                .signed_duration_since(entry.created_at)
                .num_seconds() as u64;

            if age > self.ttl_seconds {
                self.entries.remove(key);
                return None;
            }

            entry.last_accessed = Utc::now();
            entry.access_count += 1;
            Some(entry.value.clone())
        } else {
            None
        }
    }

    async fn set(&mut self, key: String, value: String, size_bytes: usize) {
        // Check if we need to evict
        while self.entries.len() >= self.max_size && !self.entries.is_empty() {
            self.evict_lru().await;
        }

        let entry = CacheEntry {
            key: key.clone(),
            value,
            size_bytes,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
        };

        self.entries.insert(key, entry);
    }

    async fn evict_lru(&mut self) {
        // Evict least recently used
        if let Some((key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
        {
            let key = key.clone();
            self.entries.remove(&key);
        }
    }

    async fn invalidate(&mut self, pattern: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|key, _| !key.starts_with(pattern));
        let removed = before - self.entries.len();

        if removed > 0 {
            println!(
                "  🗑️  {}: Invalidated {} entries matching '{}'",
                self.name, removed, pattern
            );
        }

        removed
    }

    async fn warm_cache(&mut self, keys: Vec<String>) -> usize {
        let mut warmed = 0;

        for key in keys {
            // Simulate fetching data from database
            tokio::time::sleep(Duration::from_millis(10)).await;

            let value = format!("data_for_{}", key);
            let size = value.len();

            self.set(key.clone(), value, size).await;
            warmed += 1;
        }

        println!("  🔥 {}: Warmed {} entries", self.name, warmed);
        warmed
    }

    async fn cleanup_expired(&mut self) -> usize {
        let now = Utc::now();
        let before = self.entries.len();

        self.entries.retain(|_, entry| {
            let age = now.signed_duration_since(entry.created_at).num_seconds() as u64;
            age <= self.ttl_seconds
        });

        let removed = before - self.entries.len();
        if removed > 0 {
            println!("  🧹 {}: Cleaned up {} expired entries", self.name, removed);
        }

        removed
    }

    fn get_hit_rate(&self) -> f64 {
        let total = self.entries.values().map(|e| e.access_count).sum::<u64>();

        if total == 0 {
            0.0
        } else {
            let hits: u64 = self
                .entries
                .values()
                .filter(|e| e.access_count > 0)
                .map(|e| e.access_count)
                .sum();
            hits as f64 / total as f64 * 100.0
        }
    }

    fn memory_usage(&self) -> usize {
        self.entries.values().map(|e| e.size_bytes).sum()
    }
}

impl CacheStats {
    fn calculate_overall_hit_rate(&self) -> f64 {
        let total_requests = self.l1_hits
            + self.l1_misses
            + self.l2_hits
            + self.l2_misses
            + self.cdn_hits
            + self.cdn_misses;

        if total_requests == 0 {
            return 0.0;
        }

        let total_hits = self.l1_hits + self.l2_hits + self.cdn_hits;
        (total_hits as f64 / total_requests as f64) * 100.0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init(); // Optional: enable with basic-logging feature

    println!("🗄️  Starting Cache Warming System\n");

    let scheduler = Arc::new(Scheduler::new());
    let l1_cache = Arc::new(RwLock::new(Cache::new("L1-Memory", 1000, 300))); // 5 min TTL
    let l2_cache = Arc::new(RwLock::new(Cache::new("L2-Redis", 10000, 3600))); // 1 hour TTL
    let cdn_cache = Arc::new(RwLock::new(Cache::new("CDN-Edge", 50000, 7200))); // 2 hour TTL
    let stats = Arc::new(RwLock::new(CacheStats::default()));

    let system = CacheSystem {
        scheduler: scheduler.clone(),
        l1_cache: l1_cache.clone(),
        l2_cache: l2_cache.clone(),
        cdn_cache: cdn_cache.clone(),
        stats: stats.clone(),
    };

    // Subscribe to events
    let mut event_receiver = scheduler.event_bus().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskCompleted { task_name, .. } => {
                    // Suppress for cleaner output
                }
                SchedulerEvent::TaskFailed {
                    task_name, error, ..
                } => {
                    eprintln!("❌ Cache task '{}' failed: {}", task_name, error);
                }
                _ => {}
            }
        }
    });

    setup_cache_tasks(&system).await?;

    scheduler.start().await?;

    println!("📋 Cache management tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            println!("  • {} (Tags: {:?})", task.name(), task.tags());
        }
    }
    println!("\n▶️  Running cache system (30 seconds)...\n");

    // Simulate some cache activity
    let l1_clone = l1_cache.clone();
    let stats_clone = stats.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let mut cache = l1_clone.write().await;
            let mut s = stats_clone.write().await;

            // Simulate random cache access
            if rand::random::<f64>() > 0.3 {
                if cache.get("product_123").await.is_some() {
                    s.l1_hits += 1;
                } else {
                    s.l1_misses += 1;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(30)).await;

    // Summary
    println!("\n📊 Cache System Summary:");
    let l1 = l1_cache.read().await;
    let l2 = l2_cache.read().await;
    let cdn = cdn_cache.read().await;
    let s = stats.read().await;

    println!("\n  L1 Cache (Memory):");
    println!("    Entries: {}", l1.entries.len());
    println!("    Memory: {} bytes", l1.memory_usage());

    println!("\n  L2 Cache (Redis):");
    println!("    Entries: {}", l2.entries.len());
    println!("    Memory: {} bytes", l2.memory_usage());

    println!("\n  CDN Cache (Edge):");
    println!("    Entries: {}", cdn.entries.len());

    println!("\n  Statistics:");
    println!("    L1 Hits/Misses: {}/{}", s.l1_hits, s.l1_misses);
    println!(
        "    Overall Hit Rate: {:.2}%",
        s.calculate_overall_hit_rate()
    );
    println!("    Total Warmups: {}", s.total_warmups);
    println!("    Total Evictions: {}", s.total_evictions);

    scheduler.stop().await?;
    println!("\n✅ Cache system shutdown complete");

    Ok(())
}

async fn setup_cache_tasks(system: &CacheSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: Morning cache warmup - Every day at 7 AM (before traffic spike)
    let l1 = system.l1_cache.clone();
    let l2 = system.l2_cache.clone();
    let stats = system.stats.clone();
    let morning_warmup = Task::new(move || {
        let l1_cache = l1.clone();
        let l2_cache = l2.clone();
        let s = stats.clone();
        async move {
            println!("🌅 Morning cache warmup starting...");

            // Popular products
            let popular_items = vec![
                "product_123".to_string(),
                "product_456".to_string(),
                "product_789".to_string(),
                "homepage_data".to_string(),
                "featured_products".to_string(),
            ];

            let mut l1 = l1_cache.write().await;
            let warmed_l1 = l1.warm_cache(popular_items.clone()).await;

            let mut l2 = l2_cache.write().await;
            let warmed_l2 = l2.warm_cache(popular_items).await;

            let mut stats = s.write().await;
            stats.total_warmups += (warmed_l1 + warmed_l2) as u64;

            Ok(())
        }
    })
    .with_schedule("0 7 * * *")?
    .with_tags(&["cache", "warmup", "morning"]);

    system.scheduler.add_task(morning_warmup).await?;

    // Task 2: Predictive warmup before sale - Every Friday at 11 PM
    let cdn = system.cdn_cache.clone();
    let stats = system.stats.clone();
    let sale_warmup = Task::new(move || {
        let cache = cdn.clone();
        let s = stats.clone();
        async move {
            println!("🔥 Pre-sale cache warmup...");

            let sale_items = vec![
                "sale_category_electronics".to_string(),
                "sale_category_clothing".to_string(),
                "sale_landing_page".to_string(),
                "top_deals".to_string(),
            ];

            let mut cdn_cache = cache.write().await;
            let warmed = cdn_cache.warm_cache(sale_items).await;

            let mut stats = s.write().await;
            stats.total_warmups += warmed as u64;

            Ok(())
        }
    })
    .with_schedule("0 23 * * 5")? // Friday 11 PM
    .with_tags(&["cache", "warmup", "sale"]);

    system.scheduler.add_task(sale_warmup).await?;

    // Task 3: Cleanup expired entries - Every hour
    let l1 = system.l1_cache.clone();
    let l2 = system.l2_cache.clone();
    let cdn = system.cdn_cache.clone();
    let cleanup_task = Task::new(move || {
        let l1_cache = l1.clone();
        let l2_cache = l2.clone();
        let cdn_cache = cdn.clone();
        async move {
            println!("🧹 Cleaning expired cache entries...");

            let mut l1 = l1_cache.write().await;
            l1.cleanup_expired().await;

            let mut l2 = l2_cache.write().await;
            l2.cleanup_expired().await;

            let mut cdn = cdn_cache.write().await;
            cdn.cleanup_expired().await;

            Ok(())
        }
    })
    .with_name("Cache Cleanup")
    .with_interval(Duration::from_secs(15)) // Demo: 15 seconds
    .with_tags(&["cache", "cleanup", "maintenance"]);

    system.scheduler.add_task(cleanup_task).await?;

    // Task 4: Cache invalidation after deployments - Manual trigger simulation
    let l1 = system.l1_cache.clone();
    let l2 = system.l2_cache.clone();
    let invalidation_task = Task::new(move || {
        let l1_cache = l1.clone();
        let l2_cache = l2.clone();
        async move {
            // Simulate periodic invalidation
            if rand::random::<f64>() > 0.7 {
                println!("🔄 Invalidating cache after deployment...");

                let mut l1 = l1_cache.write().await;
                l1.invalidate("product_").await;

                let mut l2 = l2_cache.write().await;
                l2.invalidate("product_").await;
            }

            Ok(())
        }
    })
    .with_name("Cache Invalidation")
    .with_interval(Duration::from_secs(20))
    .with_tags(&["cache", "invalidation"]);

    system.scheduler.add_task(invalidation_task).await?;

    // Task 5: Monitor cache hit rates - Every 5 minutes
    let l1 = system.l1_cache.clone();
    let l2 = system.l2_cache.clone();
    let stats = system.stats.clone();
    let monitoring_task = Task::new(move || {
        let l1_cache = l1.clone();
        let l2_cache = l2.clone();
        let s = stats.clone();
        async move {
            let l1 = l1_cache.read().await;
            let l2 = l2_cache.read().await;
            let stats = s.read().await;

            println!("📊 Cache Metrics:");
            println!(
                "  L1 entries: {}, Memory: {} bytes",
                l1.entries.len(),
                l1.memory_usage()
            );
            println!(
                "  L2 entries: {}, Memory: {} bytes",
                l2.entries.len(),
                l2.memory_usage()
            );
            println!(
                "  Overall hit rate: {:.2}%",
                stats.calculate_overall_hit_rate()
            );

            // Alert if hit rate is low
            if stats.calculate_overall_hit_rate() < 50.0 && stats.l1_hits + stats.l1_misses > 10 {
                println!("  ⚠️  Warning: Low cache hit rate!");
            }

            Ok(())
        }
    })
    .with_name("Cache Monitoring")
    .with_interval(Duration::from_secs(10)) // Demo: 10 seconds
    .with_tags(&["cache", "monitoring", "metrics"]);

    system.scheduler.add_task(monitoring_task).await?;

    // Task 6: Warm frequently accessed items - Every 30 minutes
    let l1 = system.l1_cache.clone();
    let stats = system.stats.clone();
    let frequent_warmup = Task::new(move || {
        let cache = l1.clone();
        let s = stats.clone();
        async move {
            println!("🔥 Warming frequently accessed items...");

            // Simulate identifying hot items
            let hot_items = vec![
                "trending_product_1".to_string(),
                "trending_product_2".to_string(),
                "popular_category".to_string(),
            ];

            let mut l1_cache = cache.write().await;
            let warmed = l1_cache.warm_cache(hot_items).await;

            let mut stats = s.write().await;
            stats.total_warmups += warmed as u64;

            Ok(())
        }
    })
    .with_name("Frequent Items Warmup")
    .with_interval(Duration::from_secs(18))
    .with_tags(&["cache", "warmup", "trending"]);

    system.scheduler.add_task(frequent_warmup).await?;

    Ok(())
}
