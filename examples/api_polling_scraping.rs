//! Real-world example: API polling and web scraping
//!
//! This example demonstrates:
//! - Periodic API polling for data updates
//! - Rate-limited web scraping
//! - Data change detection
//! - External service integration
//! - Feed aggregation

use taskline::{Scheduler, Task, TaskConfig, SchedulerEvent};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct ScraperSystem {
    scheduler: Arc<Scheduler>,
    data_store: Arc<RwLock<DataStore>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

#[derive(Default)]
struct DataStore {
    weather_data: HashMap<String, WeatherData>,
    stock_prices: HashMap<String, StockPrice>,
    rss_feeds: HashMap<String, Vec<FeedItem>>,
    product_prices: HashMap<String, ProductPrice>,
}

#[derive(Clone, Serialize, Deserialize)]
struct WeatherData {
    city: String,
    temperature: f64,
    conditions: String,
    timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct StockPrice {
    symbol: String,
    price: f64,
    change_percent: f64,
    volume: u64,
    timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct FeedItem {
    title: String,
    link: String,
    published: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ProductPrice {
    product_id: String,
    name: String,
    price: f64,
    availability: bool,
    last_checked: DateTime<Utc>,
}

struct RateLimiter {
    requests_per_minute: HashMap<String, u32>,
    max_requests: u32,
}

impl RateLimiter {
    fn new(max_requests: u32) -> Self {
        Self {
            requests_per_minute: HashMap::new(),
            max_requests,
        }
    }

    async fn can_make_request(&mut self, service: &str) -> bool {
        let count = self.requests_per_minute.entry(service.to_string()).or_insert(0);
        if *count >= self.max_requests {
            println!("⏸️  Rate limit reached for '{}', waiting...", service);
            false
        } else {
            *count += 1;
            true
        }
    }

    async fn reset(&mut self) {
        self.requests_per_minute.clear();
    }
}

impl DataStore {
    async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, String> {
        // Simulate API call
        tokio::time::sleep(Duration::from_millis(100)).await;

        let data = WeatherData {
            city: city.to_string(),
            temperature: 15.0 + (rand::random::<f64>() * 20.0),
            conditions: vec!["Sunny", "Cloudy", "Rainy", "Partly Cloudy"][rand::random::<usize>() % 4].to_string(),
            timestamp: Utc::now(),
        };

        self.weather_data.insert(city.to_string(), data.clone());
        println!("🌤️  Weather for {}: {:.1}°C, {}",
            data.city, data.temperature, data.conditions);

        Ok(data)
    }

    async fn fetch_stock_price(&mut self, symbol: &str) -> Result<StockPrice, String> {
        // Simulate API call
        tokio::time::sleep(Duration::from_millis(80)).await;

        let base_price = 100.0;
        let price = base_price + (rand::random::<f64>() * 50.0) - 25.0;
        let change = (rand::random::<f64>() * 10.0) - 5.0;

        let stock = StockPrice {
            symbol: symbol.to_string(),
            price,
            change_percent: change,
            volume: rand::random::<u64>() % 1000000,
            timestamp: Utc::now(),
        };

        // Detect significant price changes
        if let Some(prev) = self.stock_prices.get(symbol) {
            let price_change = ((stock.price - prev.price) / prev.price) * 100.0;
            if price_change.abs() > 2.0 {
                println!("📈 Alert: {} changed {:.2}% (${:.2} -> ${:.2})",
                    symbol, price_change, prev.price, stock.price);
            }
        }

        self.stock_prices.insert(symbol.to_string(), stock.clone());
        println!("💹 {}: ${:.2} ({:+.2}%)", stock.symbol, stock.price, stock.change_percent);

        Ok(stock)
    }

    async fn fetch_rss_feed(&mut self, feed_url: &str) -> Result<Vec<FeedItem>, String> {
        // Simulate fetching RSS feed
        tokio::time::sleep(Duration::from_millis(150)).await;

        let items = vec![
            FeedItem {
                title: "Breaking News: Technology Advances".to_string(),
                link: "https://example.com/article1".to_string(),
                published: Utc::now(),
            },
            FeedItem {
                title: "Market Update: Stocks Rally".to_string(),
                link: "https://example.com/article2".to_string(),
                published: Utc::now(),
            },
        ];

        // Check for new items
        let existing = self.rss_feeds.get(feed_url);
        let new_count = if let Some(existing_items) = existing {
            items.len().saturating_sub(existing_items.len())
        } else {
            items.len()
        };

        if new_count > 0 {
            println!("📰 {} new article(s) from {}", new_count, feed_url);
        }

        self.rss_feeds.insert(feed_url.to_string(), items.clone());
        Ok(items)
    }

    async fn check_product_price(&mut self, product_id: &str) -> Result<ProductPrice, String> {
        // Simulate web scraping
        tokio::time::sleep(Duration::from_millis(200)).await;

        let price = 29.99 + (rand::random::<f64>() * 20.0);
        let availability = rand::random::<f64>() > 0.2;

        let product = ProductPrice {
            product_id: product_id.to_string(),
            name: format!("Product {}", product_id),
            price,
            availability,
            last_checked: Utc::now(),
        };

        // Price drop alert
        if let Some(prev) = self.product_prices.get(product_id) {
            let price_drop = prev.price - product.price;
            if price_drop > 5.0 {
                println!("🔔 Price Drop Alert: {} dropped ${:.2} (${:.2} -> ${:.2})",
                    product.name, price_drop, prev.price, product.price);
            }
        }

        self.product_prices.insert(product_id.to_string(), product.clone());
        println!("🏷️  {}: ${:.2} {}",
            product.name, product.price,
            if product.availability { "✅" } else { "❌" });

        Ok(product)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🌐 Starting API Polling & Scraping System\n");

    let scheduler = Arc::new(Scheduler::new());
    let data_store = Arc::new(RwLock::new(DataStore::default()));
    let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(10)));

    let system = ScraperSystem {
        scheduler: scheduler.clone(),
        data_store: data_store.clone(),
        rate_limiter: rate_limiter.clone(),
    };

    // Subscribe to events
    let mut event_receiver = scheduler.event_bus().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            match event {
                SchedulerEvent::TaskCompleted { task_name, .. } => {
                    // Suppress individual task logs for cleaner output
                }
                SchedulerEvent::TaskFailed { task_name, error, .. } => {
                    eprintln!("❌ '{}' failed: {}", task_name, error);
                }
                _ => {}
            }
        }
    });

    setup_polling_tasks(&system).await?;

    scheduler.start().await?;

    println!("📋 Active polling/scraping tasks:");
    for task_id in scheduler.task_ids().await {
        if let Some(task) = scheduler.get_task(&task_id).await {
            println!("  • {} (Tags: {:?})", task.name(), task.tags());
        }
    }
    println!("\n▶️  Running tasks (30 seconds)...\n");

    // Reset rate limiter periodically
    let rate_limiter_clone = rate_limiter.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            rate_limiter_clone.write().await.reset();
        }
    });

    tokio::time::sleep(Duration::from_secs(30)).await;

    // Summary
    println!("\n📊 Data Collection Summary:");
    let store = data_store.read().await;
    println!("  Weather cities tracked: {}", store.weather_data.len());
    println!("  Stock symbols tracked: {}", store.stock_prices.len());
    println!("  RSS feeds monitored: {}", store.rss_feeds.len());
    println!("  Products tracked: {}", store.product_prices.len());

    scheduler.stop().await?;
    println!("\n✅ Scraping system shutdown complete");

    Ok(())
}

async fn setup_polling_tasks(system: &ScraperSystem) -> Result<(), Box<dyn std::error::Error>> {
    // Task 1: Weather API polling - Every 5 minutes
    let data_store = system.data_store.clone();
    let rate_limiter = system.rate_limiter.clone();
    let weather_task = Task::new(move || {
        let store = data_store.clone();
        let limiter = rate_limiter.clone();
        async move {
            let mut rl = limiter.write().await;
            if !rl.can_make_request("weather_api").await {
                return Ok(());
            }
            drop(rl);

            let mut ds = store.write().await;
            let cities = vec!["New York", "London", "Tokyo"];

            for city in cities {
                ds.fetch_weather(city).await.map_err(|e|
                    taskline::TasklineError::TaskExecutionError(e))?;
            }

            Ok(())
        }
    })
    .with_name("Weather API Poller")
    .with_interval(Duration::from_secs(10)) // Demo: 10 seconds
    .with_tags(&["api", "weather", "polling"]);

    system.scheduler.add_task(weather_task)?;

    // Task 2: Stock price monitoring - Every 1 minute
    let data_store = system.data_store.clone();
    let rate_limiter = system.rate_limiter.clone();
    let stock_task = Task::new(move || {
        let store = data_store.clone();
        let limiter = rate_limiter.clone();
        async move {
            let mut rl = limiter.write().await;
            if !rl.can_make_request("stock_api").await {
                return Ok(());
            }
            drop(rl);

            let mut ds = store.write().await;
            let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN"];

            for symbol in symbols {
                ds.fetch_stock_price(symbol).await.map_err(|e|
                    taskline::TasklineError::TaskExecutionError(e))?;
            }

            Ok(())
        }
    })
    .with_name("Stock Price Monitor")
    .with_interval(Duration::from_secs(8)) // Demo: 8 seconds
    .with_tags(&["api", "stocks", "finance", "polling"]);

    system.scheduler.add_task(stock_task)?;

    // Task 3: RSS feed aggregator - Every 15 minutes
    let data_store = system.data_store.clone();
    let rss_task = Task::new(move || {
        let store = data_store.clone();
        async move {
            let mut ds = store.write().await;
            let feeds = vec![
                "https://news.example.com/rss",
                "https://tech.example.com/feed",
            ];

            for feed in feeds {
                ds.fetch_rss_feed(feed).await.map_err(|e|
                    taskline::TasklineError::TaskExecutionError(e))?;
            }

            Ok(())
        }
    })
    .with_interval(Duration::from_secs(12)) // Demo: 12 seconds
    .with_tags(&["rss", "feed", "news"]);

    system.scheduler.add_task(rss_task)?;

    // Task 4: Product price tracker - Every 2 hours
    let data_store = system.data_store.clone();
    let rate_limiter = system.rate_limiter.clone();
    let price_tracker = Task::new(move || {
        let store = data_store.clone();
        let limiter = rate_limiter.clone();
        async move {
            let mut rl = limiter.write().await;
            if !rl.can_make_request("price_scraper").await {
                return Ok(());
            }
            drop(rl);

            let mut ds = store.write().await;
            let products = vec!["PROD001", "PROD002", "PROD003"];

            for product in products {
                ds.check_product_price(product).await.map_err(|e|
                    taskline::TasklineError::TaskExecutionError(e))?;
                tokio::time::sleep(Duration::from_millis(500)).await; // Be nice
            }

            Ok(())
        }
    })
    .with_name("Product Price Tracker")
    .with_interval(Duration::from_secs(15)) // Demo: 15 seconds
    .with_tags(&["scraping", "prices", "ecommerce"])
    .with_config(TaskConfig {
        timeout: Some(Duration::from_secs(60)),
        max_retries: 2,
        retry_delay: Duration::from_secs(10),
        fail_scheduler_on_error: false,
    });

    system.scheduler.add_task(price_tracker)?;

    // Task 5: API health check - Every 5 minutes
    let health_check = Task::new(|| async {
        println!("🏥 Checking API endpoints health...");

        let endpoints = vec![
            ("Weather API", "https://api.weather.com/health"),
            ("Stock API", "https://api.stocks.com/health"),
            ("News Feed", "https://news.example.com/health"),
        ];

        for (name, _endpoint) in endpoints {
            // Simulate health check
            tokio::time::sleep(Duration::from_millis(50)).await;
            let is_healthy = rand::random::<f64>() > 0.05; // 95% uptime

            if is_healthy {
                println!("  ✅ {} is healthy", name);
            } else {
                println!("  ⚠️  {} is experiencing issues", name);
            }
        }

        Ok(())
    })
    .with_name("API Health Check")
    .with_interval(Duration::from_secs(20))
    .with_tags(&["monitoring", "health", "api"]);

    system.scheduler.add_task(health_check)?;

    Ok(())
}
