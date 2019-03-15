use std::mem;
use std::collections::VecDeque;
use mio::{Registration, Ready, SetReadiness};
use crate::errors::Result;

use super::{Reactor, Reaction, EventedReactor};

/// The [`ReactiveGenerator`] reacts as soon as the [`System`] starts.
/// ```
/// # use sonr::system::{System, SystemEvent};
/// # use sonr::reactor::Reactor;
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


impl<T> Reactor for ReactiveGenerator<T> {
    type Output = T;
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(ev) = reaction {
            if ev.token() != self.reactor.token() {
                return Reaction::Event(ev)
            }

            if let Some(val) = self.inner.pop_front() {
                return Reaction::Value(val);
            }
        }

        if let Reaction::Continue = reaction {
            if let Some(val) = self.inner.pop_front() {
                return Reaction::Value(val);
            }
        }

        match reaction {
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(_) => Reaction::Continue,
        }
    }
}

/// A [`Mono`] reacts as soon as the [`System`] starts and produces exactly one value
/// (the value set by the constructor).
/// ```
/// # use sonr::system::{System, SystemEvent};
/// # use sonr::reactor::Reactor;
/// # use sonr::reactor::producers::Mono;
/// fn main() {
///     let handle = System::init().unwrap();
///     let gen = Mono::new(0u8).unwrap();
///
///     System::start(gen.map(|i| {
///         assert_eq!(i, 0u8);
///         handle.send(SystemEvent::Stop) 
///     }));
/// }
/// ```
///
/// [`Mono`]: struct.Mono.html
/// [`System`]: ../../system/struct.System.html
pub struct Mono<T> {
    inner: Reaction<T>,
    reactor: EventedReactor<Registration>,
}

impl<T> Mono<T> {
    pub fn new(val: T) -> Result<Self> {
        let (reg, set_ready) = Registration::new2();
        let reactor = EventedReactor::new(reg, Ready::readable())?;
        set_ready.set_readiness(Ready::readable())?;
        Ok(Self { 
            inner: Reaction::Value(val),
            reactor,
        })
    }
}

impl<T> Reactor for Mono<T> {
    type Input = ();
    type Output = T;

    fn react(&mut self, reaction: Reaction<()>) -> Reaction<Self::Output> {
        if let Reaction::Event(ev) = reaction {
            if ev.token() != self.reactor.token() {
                return ev.into()
            }

            let mut output = Reaction::Continue;
            mem::swap(&mut self.inner, &mut output);
            return output;
        }

        match reaction {
            Reaction::Continue => Reaction::Continue,
            Reaction::Event(ev) => Reaction::Event(ev),
            Reaction::Value(_) => Reaction::Continue,
        }
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
/// }));
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
        //set_ready.set_readiness(Ready::readable())?;
        Ok(Self { 
            reactor,
            reaction: Reaction::Continue,
            set_ready,
        })
    }
}

impl<T> Reactor for ReactiveConsumer<T> {
    type Output = T;
    type Input = T;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        reaction
    }
}
