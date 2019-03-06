use std::cell::RefCell;

use mio::{Evented, Events, Poll, Token, Ready, PollOpt};

use crate::PreVec;
use crate::sync::signal::{SignalReceiver, SignalSender};
use crate::errors::Result;

use super::reactor::{Reactive, EventedReactor, Reaction};

thread_local! {
    static CURRENT_SYSTEM: RefCell<Option<System>> = RefCell::new(None);
}

/// Specific event the `System` responds to
/// NOTE: There should only be one `System::init` call per thread
/// The System handles registration and pushes `Event`s to the reactors
/// passed to `System::start`.
#[derive(Debug)]
pub enum SystemEvent {
    Stop,
    Dummy
}

pub struct System {
    reactors: PreVec<()>,
    poll: Poll,
    rx: SignalReceiver<SystemEvent>,
}

static SERVER_TOKEN: Token = Token(0);

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
            SERVER_TOKEN,
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
            assert!(current.is_none(), "System already initialised");
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
    pub fn start<R: Reactive>(mut reactor: R) -> Result<()> {
        let mut events = Events::with_capacity(1024);

        'system: loop {
            with_system!(current, { current.poll.poll(&mut events, None) })?;

            for event in &events {
                if event.token() == SERVER_TOKEN { 
                    let sys_events = with_system!(current, {
                        let mut sys_events = Vec::new();
                        if let Ok(sys_event) = current.rx.try_recv() {
                            sys_events.push(sys_event);
                        }
                        sys_events
                    });

                    eprintln!("{:?}", sys_events);
                    for sys_event in sys_events {
                        match sys_event {
                            SystemEvent::Stop => break 'system,
                            SystemEvent::Dummy => {
                                // TODO: remove this
                                eprintln!("{:?}", "received DUMMY event");
                            }
                        }
                    }
                } else {
                    let reaction = reactor.react(Reaction::Event(event));

                    if let Reaction::Stream(_) = reaction {
                        while let Reaction::Stream(_) = reactor.react(Reaction::NoReaction) { }
                    } 

                    // TODO: remove this, remnants from old
                    // reactor.react(Reaction::Event(event));
                    // while let Reaction::Value(_) = reactor.react(Reaction::NoReaction) { }
                    // if let Reaction::Value(_) = reaction {
                    //     while let Reaction::Value(_) = reactor.react(Reaction::NoReaction) { }
                    // }

                    // TODO: check while
                    // while let Reaction::Value(_) = reactor.react(Reaction::Event(event)) {}
                    // match x {
                    //     Reaction::NoReaction => eprintln!("{:?}", "NoReaction"),
                    //     Reaction::Value(val) => eprintln!("{:?}", "Value"),
                    //     Reaction::Event(event) => eprintln!("{:?}", "Event"),
                    // }
                    //while let Reaction::Value(_) = reactor.react() { }
                }
            }
        }

        Ok(())
    } 

    pub fn free_token(token: Token) {
        with_system!(current, { current.reactors.remove(token.0); });
    }

    pub fn reserve_token() -> Result<Token> {
        with_system!(current, { 
            let token = current.reactors.insert(())?;
            Ok(Token(token))
        })
    } 

    /// Send a system event to the current system.
    pub fn send(sys_event: SystemEvent) {
        with_system!(current, { current.rx.sender().send(sys_event) });
    } 
}
