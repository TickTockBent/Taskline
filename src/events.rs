//! Event system for task and scheduler events.
//!
//! This module provides an event bus for subscribing to task and scheduler lifecycle events.

use crate::task::TaskStatus;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Represents different types of events that can occur in the scheduler.
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// The scheduler has started
    SchedulerStarted { timestamp: DateTime<Utc> },

    /// The scheduler has stopped
    SchedulerStopped {
        timestamp: DateTime<Utc>,
        uptime_seconds: i64,
    },

    /// A task was added to the scheduler
    TaskAdded {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },

    /// A task was removed from the scheduler
    TaskRemoved {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },

    /// A task is about to start execution
    TaskStarting {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },

    /// A task has completed successfully
    TaskCompleted {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
        duration_ms: u64,
    },

    /// A task has failed
    TaskFailed {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
        error: String,
        retry_count: u32,
    },

    /// A task has timed out
    TaskTimedOut {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
        timeout_duration_ms: u64,
    },

    /// A task is approaching its timeout (warning at 80%)
    TaskTimeoutWarning {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
        percent_complete: u8,
    },

    /// A task was cancelled
    TaskCancelled {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },

    /// A task's status changed
    TaskStatusChanged {
        task_id: String,
        task_name: String,
        old_status: TaskStatus,
        new_status: TaskStatus,
        timestamp: DateTime<Utc>,
    },

    /// A task was paused
    TaskPaused {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },

    /// A task was resumed
    TaskResumed {
        task_id: String,
        task_name: String,
        timestamp: DateTime<Utc>,
    },
}

/// Event bus for broadcasting scheduler and task events.
///
/// The event bus allows subscribers to receive notifications about
/// task execution, scheduler lifecycle, and other important events.
///
/// # Examples
///
/// ```
/// use taskline::events::{EventBus, SchedulerEvent};
///
/// #[tokio::main]
/// async fn main() {
///     let event_bus = EventBus::new();
///
///     let mut receiver = event_bus.subscribe();
///
///     tokio::spawn(async move {
///         while let Ok(event) = receiver.recv().await {
///             println!("Received event: {:?}", event);
///         }
///     });
/// }
/// ```
#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<SchedulerEvent>>,
}

impl EventBus {
    /// Creates a new event bus with a default capacity of 1000 events.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::events::EventBus;
    ///
    /// let event_bus = EventBus::new();
    /// ```
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Creates a new event bus with a specified capacity.
    ///
    /// The capacity determines how many events can be buffered before
    /// old events are dropped if receivers are slow.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of events to buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::events::EventBus;
    ///
    /// let event_bus = EventBus::with_capacity(500);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        EventBus {
            sender: Arc::new(sender),
        }
    }

    /// Publishes an event to all subscribers.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::events::{EventBus, SchedulerEvent};
    /// use chrono::Utc;
    ///
    /// let event_bus = EventBus::new();
    /// event_bus.publish(SchedulerEvent::SchedulerStarted {
    ///     timestamp: Utc::now(),
    /// });
    /// ```
    pub fn publish(&self, event: SchedulerEvent) {
        // Ignore errors - no subscribers is fine
        let _ = self.sender.send(event);
    }

    /// Subscribes to events from this event bus.
    ///
    /// Returns a receiver that can be used to receive events.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::events::EventBus;
    ///
    /// let event_bus = EventBus::new();
    /// let mut receiver = event_bus.subscribe();
    ///
    /// // In an async context:
    /// // while let Ok(event) = receiver.recv().await {
    /// //     println!("Event: {:?}", event);
    /// // }
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<SchedulerEvent> {
        self.sender.subscribe()
    }

    /// Returns the number of active subscribers.
    ///
    /// # Examples
    ///
    /// ```
    /// use taskline::events::EventBus;
    ///
    /// let event_bus = EventBus::new();
    /// assert_eq!(event_bus.subscriber_count(), 0);
    ///
    /// let _receiver = event_bus.subscribe();
    /// assert_eq!(event_bus.subscriber_count(), 1);
    /// ```
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let event_bus = EventBus::new();
        assert_eq!(event_bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_bus_subscribe() {
        let event_bus = EventBus::new();
        let _receiver = event_bus.subscribe();
        assert_eq!(event_bus.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_event_bus_publish_receive() {
        let event_bus = EventBus::new();
        let mut receiver = event_bus.subscribe();

        let event = SchedulerEvent::SchedulerStarted {
            timestamp: Utc::now(),
        };

        event_bus.publish(event.clone());

        let received = receiver.recv().await.unwrap();
        match received {
            SchedulerEvent::SchedulerStarted { .. } => (),
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = EventBus::new();
        let mut receiver1 = event_bus.subscribe();
        let mut receiver2 = event_bus.subscribe();

        assert_eq!(event_bus.subscriber_count(), 2);

        let event = SchedulerEvent::TaskAdded {
            task_id: "test-123".to_string(),
            task_name: "Test Task".to_string(),
            timestamp: Utc::now(),
        };

        event_bus.publish(event);

        // Both receivers should get the event
        let _ = receiver1.recv().await.unwrap();
        let _ = receiver2.recv().await.unwrap();
    }

    #[test]
    fn test_event_bus_with_capacity() {
        let event_bus = EventBus::with_capacity(100);
        assert_eq!(event_bus.subscriber_count(), 0);
    }
}
