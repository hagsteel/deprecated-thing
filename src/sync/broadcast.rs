use std::sync::{Arc, Mutex};

use mio::Event;

use crate::sync::signal::{SignalReceiver, SignalSender};
use crate::reactor::{Reaction, Reactive};

use super::Capacity; 

// -----------------------------------------------------------------------------
//              - Broadcast -
//              notify every subscriber, meaning T has to be Clone
// -----------------------------------------------------------------------------
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
    pub fn unbounded() -> Self {
        Self::from(Capacity::Unbounded)
    }

    pub fn bounded(capacity: usize) -> Self {
        Self::from(Capacity::Bounded(capacity))
    }

    // Register a signal trigger that will listen to 
    // broadcasts from this broadcaster
    pub fn subscriber(&self) -> SignalReceiver<T> {
        let signal = SignalReceiver::from(&self.capacity);
        let trigger = signal.sender();
        if let Ok(ref mut subs) = self.subscribers.lock() {
            subs.push(trigger);
        }
        signal
    }

    pub fn publish(&self, val: T) {
        match self.subscribers.lock() {
            Ok(subs) => {
                for sub in subs.iter() {
                    let val_c = val.clone();
                    sub.send(val_c);
                }
            }
            Err(e) => { eprintln!("mutex error: {:?}", e); }
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
pub struct ReactiveBroadcast<T: Clone> {
    inner: Broadcast<T>,
}

impl<T: Clone> Reactive for ReactiveBroadcast<T> {
    type Output = ();
    type Input = T;

    // Isn't Evented so doesn't react
    fn reacting(&mut self, _: Event) -> bool { false }

    fn react(&mut self) -> Reaction<Self::Output> { Reaction::NoReaction }
    fn react_to(&mut self, input: Self::Input) {
        self.inner.publish(input)
    }
}
