//! Broadcast 
use std::sync::{Arc, Mutex};

use crate::sync::signal::{SignalReceiver, SignalSender};
use crate::reactor::{Reaction, Reactor};

use super::Capacity; 

// -----------------------------------------------------------------------------
//              - Broadcast -
//              notify every subscriber, meaning T has to be Clone
// -----------------------------------------------------------------------------
/// Broadcast value to all subscribers.
///
/// This is useful in a pub/sub setup, however it requires that each value implements
/// clone as the data is cloned.
pub struct Broadcast<T: Clone> {
    subscribers: Arc<Mutex<Vec<SignalSender<T>>>>,
    capacity: Capacity,
}

impl<T: Clone> From<Capacity> for Broadcast<T> {
    fn from(capacity: Capacity) -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            capacity,
        }
    }
}

impl<T: Clone> Broadcast<T> {
    /// Create an unbounded broadcaster
    pub fn unbounded() -> Self {
        Self::from(Capacity::Unbounded)
    }

    /// Create an bounded broadcaster
    pub fn bounded(capacity: usize) -> Self {
        Self::from(Capacity::Bounded(capacity))
    }

    /// Create a new subscriber of the data
    pub fn subscriber(&self) -> SignalReceiver<T> {
        let signal = SignalReceiver::from(&self.capacity);
        let trigger = signal.sender();
        if let Ok(ref mut subs) = self.subscribers.lock() {
            subs.push(trigger);
        }
        signal
    }

    /// Publish data to all subscribers.
    /// Note that the published data is cloned for each subscriber.
    pub fn publish(&self, val: T) {
        match self.subscribers.lock() {
            Ok(subs) => {
                for sub in subs.iter() {
                    let val_c = val.clone();
                    let _ = sub.send(val_c);
                }
            }
            Err(_e) => { /* Mutex error: ignored for now */ }
        }

    }
}

impl<T: Clone> Clone for Broadcast<T> {
    fn clone(&self) -> Self {
        Self {
            subscribers: self.subscribers.clone(),
            capacity: self.capacity
        }
    }
}

// -----------------------------------------------------------------------------
// 		- Reactive broadcast -
// -----------------------------------------------------------------------------
/// A reactive broadcaster
pub struct ReactiveBroadcast<T: Clone> {
    inner: Broadcast<T>,
}

impl<T: Clone> Reactor for ReactiveBroadcast<T> {
    type Output = ();
    type Input = T;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value(val) => {
                self.inner.publish(val);
                Reaction::Value(())
            },
            Reaction::Event(e) => Reaction::Event(e),
            Reaction::Continue => Reaction::Continue,
        }
    }
}
