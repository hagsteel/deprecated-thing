use std::collections::VecDeque;
use mio::{Event, Registration, Ready};
use crate::errors::Result;

use super::{Reactive, Reaction, EventedReactor};

/// The [`EventedGenerator`] reacts as soon as the [`System`] starts.
/// ```
/// # use sonr::system::{System, SystemEvent};
/// # use sonr::reactor::Reactive;
/// # use sonr::reactor::producers::EventedGenerator;
/// fn main() {
///     let handle = System::init().unwrap();
///     let gen = EventedGenerator::new(vec![0u8]).unwrap();
///
///     System::start(gen.map(|i| {
///         assert_eq!(i, 0u8);
///         handle.send(SystemEvent::Stop) 
///     }));
/// }
/// ```
///
/// [`EventedGenerator`]: struct.EventedGenerator.html
/// [`System`]: ../../system/struct.System.html
pub struct EventedGenerator<T> {
    inner: VecDeque<T>,
    reactor: EventedReactor<Registration>,
}

impl<T> EventedGenerator<T> {
    pub fn new(inner: Vec<T>) -> Result<Self> {
        let (reg, set_ready) = Registration::new2();
        let reactor = EventedReactor::new(reg, Ready::readable())?;
        set_ready.set_readiness(Ready::readable())?;
        Ok(Self { 
            inner: inner.into(),
            reactor,
        })
    }
}


impl<T> Reactive for EventedGenerator<T> {
    type Output = T;
    type Input = T;

    fn reacting(&mut self, event: Event) -> bool {
        self.reactor.token() == event.token()
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        match self.inner.pop_front() {
            Some(val) => Reaction::Value(val),
            None => Reaction::NoReaction,
        }
    }

    fn react_to(&mut self, input: Self::Input) {
        self.inner.push_back(input)
    }
}
