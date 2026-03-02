//! Channel primitives for message passing
//!
//! Provides bounded and unbounded channels for sending messages between tasks.
//! Channels are the primary mechanism for inter-task communication.
//!
//! Architecture: Uses std::sync::mpsc for thread-safe message passing.
//! `receive()` returns a PENDING future resolved by a background thread,
//! enabling real concurrency when combined with `spawn()`.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Sender half of a channel
pub struct ChannelSender {
    inner: Arc<std::sync::mpsc::Sender<Value>>,
    capacity: Option<usize>,
}

impl ChannelSender {
    /// Send a value through the channel
    ///
    /// Returns true if the message was sent, false if channel is closed.
    pub fn send(&self, value: Value) -> bool {
        self.inner.send(value).is_ok()
    }

    /// Check if channel is closed (by trying a dummy operation)
    pub fn is_closed(&self) -> bool {
        // std::sync::mpsc doesn't have is_closed; approximate by checking
        // if the receiver has been dropped
        false // Conservative: assume open
    }

    /// Get channel capacity (None for unbounded)
    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }
}

impl Clone for ChannelSender {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: self.capacity,
        }
    }
}

impl fmt::Debug for ChannelSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelSender")
            .field("capacity", &self.capacity)
            .finish()
    }
}

/// Receiver half of a channel
pub struct ChannelReceiver {
    inner: Arc<Mutex<std::sync::mpsc::Receiver<Value>>>,
}

impl ChannelReceiver {
    /// Receive a value from the channel
    ///
    /// Returns a PENDING future. A background thread blocks on the channel
    /// and resolves the future when a message arrives.
    pub fn receive(&self) -> AtlasFuture {
        let receiver = Arc::clone(&self.inner);
        let future = AtlasFuture::new_pending();
        let future_clone = future.clone();

        std::thread::spawn(move || {
            let rx = receiver.lock().unwrap();
            match rx.recv() {
                Ok(value) => future_clone.resolve(value),
                Err(_) => future_clone.reject(Value::string("Channel closed")),
            }
        });

        future
    }

    /// Try to receive a value without blocking
    ///
    /// Returns Some(value) if a message is immediately available,
    /// None if the channel is empty.
    pub fn try_receive(&self) -> Option<Value> {
        let rx = self.inner.lock().unwrap();
        rx.try_recv().ok()
    }
}

impl Clone for ChannelReceiver {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Debug for ChannelReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelReceiver").finish()
    }
}

/// Create a new unbounded channel
///
/// Returns (sender, receiver) pair.
pub fn channel_unbounded() -> (ChannelSender, ChannelReceiver) {
    let (tx, rx) = std::sync::mpsc::channel();

    let sender = ChannelSender {
        inner: Arc::new(tx),
        capacity: None,
    };

    let receiver = ChannelReceiver {
        inner: Arc::new(Mutex::new(rx)),
    };

    (sender, receiver)
}

/// Create a new bounded channel
///
/// Returns (sender, receiver) pair.
/// Note: uses unbounded internally but tracks capacity for metadata.
pub fn channel_bounded(capacity: usize) -> (ChannelSender, ChannelReceiver) {
    let (tx, rx) = std::sync::mpsc::channel();

    let sender = ChannelSender {
        inner: Arc::new(tx),
        capacity: Some(capacity),
    };

    let receiver = ChannelReceiver {
        inner: Arc::new(Mutex::new(rx)),
    };

    (sender, receiver)
}

/// Select from multiple channel receivers
///
/// Returns a PENDING future that resolves with [value, index] when any
/// channel has a message. A background thread polls all receivers.
pub fn channel_select(receivers: Vec<ChannelReceiver>) -> AtlasFuture {
    if receivers.is_empty() {
        return AtlasFuture::rejected(Value::string("No channels to select from"));
    }

    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    // Collect Arc clones of receivers
    let receiver_arcs: Vec<_> = receivers.iter().map(|r| Arc::clone(&r.inner)).collect();

    std::thread::spawn(move || {
        loop {
            for (idx, receiver) in receiver_arcs.iter().enumerate() {
                let rx = receiver.lock().unwrap();
                if let Ok(value) = rx.try_recv() {
                    future_clone.resolve(Value::array(vec![value, Value::Number(idx as f64)]));
                    return;
                }
            }
            // Small yield before retrying
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    future
}
