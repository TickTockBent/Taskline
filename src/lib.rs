// Taskline: Lightweight Task Scheduling for Rust
//
// A minimalist yet powerful cron-style task scheduler built with Rust.
// Designed for simplicity, performance, and ease of integration.

// Re-export the main components
pub use crate::scheduler::Scheduler;
pub use crate::task::Task;
pub use crate::errors::TasklineError;

// Main modules
pub mod scheduler;
pub mod task;
pub mod errors;
mod cron_parser;

// Export types for convenience
pub type Result<T> = std::result::Result<T, TasklineError>;

// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");