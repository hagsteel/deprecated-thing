use std::mem;
use std::collections::VecDeque;
use mio::{Event, Registration, Ready, SetReadiness};
use crate::errors::Result;

use super::{Reactive, Reaction, EventedReactor};

/// The [`ReactiveGenerator`] reacts as soon as the [`System`] starts.
/// ```
/// # use sonr::system::{System, SystemEvent};
/// # use sonr::reactor::Reactive;
/// # use sonr::reactor::producers::ReactiveGenerator;
/// fn main() {
///     let handle = System::init().unwrap();
///     let gen = ReactiveGenerator::new(vec![0u8]).unwrap();
///
///     System::start(gen.map(|i| {
///         assert_eq!(i, 0u8);
///         handle.send(SystemEvent::Stop) 
///     }));
/// }
/// ```
///
/// [`ReactiveGenerator`]: struct.ReactiveGenerator.html
/// [`System`]: ../../system/struct.System.html
pub struct ReactiveGenerator<T> {
    inner: VecDeque<T>,
    reactor: EventedReactor<Registration>,
}

impl<T> ReactiveGenerator<T> {
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


impl<T> Reactive for ReactiveGenerator<T> {
    type Output = T;
    type Input = T;

    fn reacting(&mut self, event: Event) -> bool {
        self.reactor.token() == event.token()
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        self.inner.pop_front().into()
    }

    fn react_to(&mut self, input: Self::Input) {
        self.inner.push_back(input)
    }
}

/// A [`ReactiveConsumer`] 
/// ```
/// # use sonr::reactor::producers::{ReactiveGenerator, ReactiveConsumer};
/// # use sonr::prelude::*;
/// # use sonr::errors::Result;
/// # fn main() -> Result<()> {
/// let handle = System::init()?;
/// let numbers = ReactiveGenerator::new(vec![1, 2, 3])?;
/// let consumer = ReactiveConsumer::new()?;
///
/// let run = numbers.chain(consumer.map(|num| {
///     handle.send(SystemEvent::Stop);
/// }).noop());
///
/// System::start(run);
/// # Ok(())
/// # }
/// ```
pub struct ReactiveConsumer<T> {
    reactor: EventedReactor<Registration>,
    reaction: Reaction<T>,
    set_ready: SetReadiness,
}

impl<T> ReactiveConsumer<T> {
    pub fn new() -> Result<Self> {
        let (reg, set_ready) = Registration::new2();
        let reactor = EventedReactor::new(reg, Ready::readable())?;
        set_ready.set_readiness(Ready::readable())?;
        Ok(Self { 
            reactor,
            reaction: Reaction::NoReaction,
            set_ready,
        })
    }
}

impl<T> Reactive for ReactiveConsumer<T> {
    type Output = T;
    type Input = T;

    fn reacting(&mut self, event: Event) -> bool {
        let is = self.reactor.token() == event.token();
        if is {
            eprintln!("{:?}", "reacting");
        }
        is
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        let mut output = Reaction::NoReaction;
        mem::swap(&mut self.reaction, &mut output);
        let _ = self.set_ready.set_readiness(Ready::readable());
        output
    }

    fn react_to(&mut self, input: Self::Input) {
        eprintln!("{:?}", "reacting to");
        let _ = self.set_ready.set_readiness(Ready::readable());
        self.reaction = Reaction::Value(input);
    }
}
