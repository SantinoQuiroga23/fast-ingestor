//! Bounded in-memory queue with backpressure support.
//!
//! Uses a fixed-capacity channel so that RAM usage stays within
//! a predictable budget. When the queue is full, producers receive
//! an immediate error instead of blocking.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

/// Configuration for memory-based backpressure.
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum bytes the queue is allowed to hold.
    pub max_buffer_bytes: usize,
    /// Estimated bytes per record.
    pub estimated_record_bytes: usize,
}

impl BackpressureConfig {
    /// Calculates the channel capacity from the memory budget.
    ///
    /// Always returns at least 1 to avoid zero-capacity channels.
    pub fn channel_capacity(&self) -> usize {
        let per_item = self.estimated_record_bytes.max(1);
        (self.max_buffer_bytes / per_item).max(1)
    }
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            // 100 MB default budget
            max_buffer_bytes: 100 * 1024 * 1024,
            // Conservative default; override with real estimate
            estimated_record_bytes: 256,
        }
    }
}

/// Thread-safe bounded queue with non-blocking push.
pub struct BoundedQueue<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    /// Creates a queue with an explicit item capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let (tx, rx) = bounded(capacity);
        Self { tx, rx, capacity }
    }

    /// Creates a queue whose capacity is derived from a memory budget.
    pub fn with_memory_budget(config: &BackpressureConfig) -> Self {
        Self::with_capacity(config.channel_capacity())
    }

    /// Attempts to push an item without blocking.
    ///
    /// Returns `false` when the queue is full.
    pub fn try_push(&self, item: T) -> Result<(), BoundedQueueError> {
        match self.tx.try_send(item) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(BoundedQueueError::Full),
            Err(TrySendError::Disconnected(_)) => {
                Err(BoundedQueueError::Disconnected)
            }
        }
    }

    /// Drains all currently available items without blocking.
    pub fn drain(&self) -> Vec<T> {
        self.rx.try_iter().collect()
    }

    /// Approximate number of items currently queued.
    pub fn len(&self) -> usize {
        self.rx.len()
    }

    /// Whether the queue appears empty.
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// The maximum number of items this queue can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Errors produced by bounded queue operations.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BoundedQueueError {
    #[error("queue is full")]
    Full,

    #[error("queue receiver was dropped")]
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_drain_preserves_order() {
        let queue = BoundedQueue::with_capacity(4);
        queue.try_push(1).unwrap();
        queue.try_push(2).unwrap();
        queue.try_push(3).unwrap();

        let drained = queue.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn rejects_when_full() {
        let queue = BoundedQueue::with_capacity(2);
        queue.try_push("a").unwrap();
        queue.try_push("b").unwrap();

        let result = queue.try_push("c");
        assert_eq!(result.unwrap_err(), BoundedQueueError::Full);
    }

    #[test]
    fn space_frees_after_drain() {
        let queue = BoundedQueue::with_capacity(1);
        queue.try_push(1).unwrap();
        assert!(queue.try_push(2).is_err());

        queue.drain();
        assert!(queue.try_push(3).is_ok());
    }

    #[test]
    fn capacity_from_memory_budget() {
        let config = BackpressureConfig {
            max_buffer_bytes: 1024,
            estimated_record_bytes: 256,
        };
        assert_eq!(config.channel_capacity(), 4);
    }

    #[test]
    fn capacity_never_zero() {
        let config = BackpressureConfig {
            max_buffer_bytes: 0,
            estimated_record_bytes: 256,
        };
        assert_eq!(config.channel_capacity(), 1);
    }

    #[test]
    fn zero_item_size_does_not_panic() {
        let config = BackpressureConfig {
            max_buffer_bytes: 1024,
            estimated_record_bytes: 0,
        };
        // Should not divide by zero
        assert_eq!(config.channel_capacity(), 1024);
    }

    #[test]
    fn memory_budget_creates_working_queue() {
        let config = BackpressureConfig {
            max_buffer_bytes: 512,
            estimated_record_bytes: 128,
        };
        let queue = BoundedQueue::with_memory_budget(&config);
        assert_eq!(queue.capacity(), 4);

        for i in 0..4 {
            queue.try_push(i).unwrap();
        }
        assert!(queue.try_push(99).is_err());
    }

    #[test]
    fn len_reflects_items() {
        let queue = BoundedQueue::with_capacity(10);
        assert_eq!(queue.len(), 0);

        queue.try_push(1).unwrap();
        queue.try_push(2).unwrap();
        assert_eq!(queue.len(), 2);

        queue.drain();
        assert_eq!(queue.len(), 0);
    }
}