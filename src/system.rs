//! The `System` handles polling events, registering evented reactors and token management.
//!
//! When [`init`] is called, the [`System`] is setup for the local thread.
//!
//! Once [`start`] is called with a reactor the system begins to poll [`Event`]s.
//!
//! When an [`Event`] becomes available the [`Reactor`]s [`react`] functions is called with 
//! [`Reaction::Event(event)`] and starts propagating the [`Reaction`]s.
//!
//!
//! [`Event`]: ../struct.Event.html
//! [`Token`]: ../struct.Token.html
//! [`System`]: struct.System.html
//! [`init`]: struct.System.html#method.init
//! [`start`]: struct.System.html#method.start
//! [`Reactor`]: ../reactor/trait.Reactor.html
//! [`react`]: ../reactor/trait.Reactor.html#tymethod.react
//! [`Reaction::Event(event)`]: ../reactor/enum.Reaction.html
//!
use std::cell::RefCell;

use mio::{Evented, Events, Poll, Token, Ready, PollOpt};

use crate::PreVec;
use crate::sync::signal::{SignalReceiver, SignalSender};
use crate::errors::Result;

use super::reactor::{Reactor, EventedReactor, Reaction};

thread_local! {
    static CURRENT_SYSTEM: RefCell<Option<System>> = RefCell::new(None);
}

/// Specific event the [`System`] responds to
/// NOTE: There should only be one [`System::init`] call per thread
/// The System handles registration and pushes `Event`s to the reactors
/// passed to [`System::start`].
///
/// [`System`]: struct.System.html
/// [`System::start`]: struct.System.html#method.start
/// [`System::init`]: struct.System.html#method.init
#[derive(Debug)]
pub enum SystemEvent {
    /// Stop the System
    Stop,
}

/// `System` is thread local and has to exist for each thread using `Reactors` that react to
/// [`Reaction::Event(event)`].
///  There can only be **one** instance of a `System` per thread, and this instance is created by
///  calling `System::init()`.
///
/// [`System::init()`]: struct.System.html#method.init
/// [`Reaction::Event(event)`]: ../reactor/enum.Reaction.html
/// [`System::start()`]: struct.System.html#method.start
pub struct System {
    reactors: PreVec<()>,
    poll: Poll,
    rx: SignalReceiver<SystemEvent>,
}

static SYSTEM_TOKEN: Token = Token(0);

macro_rules! with_system {
    ($cu:ident, $x:block) => (
        {
            CURRENT_SYSTEM.with(|cell| match *cell.borrow_mut() {
                Some(ref mut $cu) => {
                    $x
                }
                None => panic!("System was not started")
            })
        }
    )
}

impl System {
    fn new() -> Result<Self> {
        let capacity = 100_000;
        let rx = SignalReceiver::unbounded();
        let poll = Poll::new()?;

        let mut reactors = PreVec::with_capacity(capacity);
        reactors.insert(())?; // Reserve the first token as it's the SERVER_TOKEN

        poll.register(
            &rx,
            SYSTEM_TOKEN,
            Ready::readable(),
            PollOpt::edge()
        )?;

        Ok(Self { 
            reactors,
            poll,
            rx,
        })
    }

    /// Initialise the system for the current thread.
    /// Should only be called once per thread.
    pub fn init() -> Result<SignalSender<SystemEvent>> {
        CURRENT_SYSTEM.with(|cell| {
            let mut current = cell.borrow_mut();
            if let Some(ref mut c) = *current {
                return Ok(c.rx.sender());
            }
            let system = Self::new()?;
            let handle = system.rx.sender();
            *current = Some(system);
            Ok(handle)
        })
    }

    /// Register an `Evented` with the System.
    pub fn register(evented: &impl Evented, interest: Ready, token: Token) -> Result<()> { 
        with_system! (current, {
            current.poll.register(
                evented,
                token,
                interest,
                PollOpt::edge()
            )?;
            Ok(())
        })
    }

    /// Reregister an evented reactor.
    pub fn reregister<T: Evented>(evented: &EventedReactor<T>) -> Result<()> {
        with_system! (current, {
            current.poll.reregister(
                evented.inner(),
                evented.token(),
                evented.interest(),
                PollOpt::edge()
            )?;
            Ok(())
        })
    }

    /// Start the event loop.
    /// This will run until `SystemEvent::Stop` is sent to the system's `SignalReceiver`.
    ///
    /// The `SignalReceiver` is returned from `System::init()`.
    pub fn start<R: Reactor>(mut reactor: R) -> Result<()> {
        let mut events = Events::with_capacity(1024);

        'system: loop {
            with_system!(current, { current.poll.poll(&mut events, None) })?;

            for event in &events {
                if event.token() == SYSTEM_TOKEN { 
                    let sys_events = with_system!(current, {
                        let mut sys_events = Vec::new();
                        if let Ok(sys_event) = current.rx.try_recv() {
                            sys_events.push(sys_event);
                        }
                        sys_events
                    });

                    for sys_event in sys_events {
                        match sys_event {
                            SystemEvent::Stop => break 'system,
                        }
                    }
                } else {
                    let reaction = reactor.react(Reaction::Event(event));

                    if let Reaction::Value(_) = reaction {
                        while let Reaction::Value(_) = reactor.react(Reaction::Continue) { }
                    } 
                }
            }
        }

        Ok(())
    } 

    /// The token can be registered with another reactor.
    /// This is called when an [`EventedReactor`] is dropped.
    ///
    /// [`EventedReactor`]: ../reactor/struct.EventedReactor.html
    pub fn free_token(token: Token) {
        let _ = with_system!(current, { current.reactors.remove(token.0); });
    }

    /// Reserve a token
    pub fn reserve_token() -> Result<Token> {
        with_system!(current, { 
            let token = current.reactors.insert(())?;
            Ok(Token(token))
        })
    } 

    /// Send a system event to the current system.
    pub fn send(sys_event: SystemEvent) {
        let _ = with_system!(current, { current.rx.sender().send(sys_event) });
    } 
}
