//! Reactors are the heart of Sonr and work by pushing data down a chain of reactors.
//!
//!```no_run
//! use std::io::Write;
//! use std::thread;
//! use sonr::prelude::*;
//! use sonr::errors::Result;
//! use sonr::sync::queue::{ReactiveQueue, ReactiveDeque};
//! use sonr::net::tcp::{ReactiveTcpListener, TcpStream, ReactiveTcpStream};
//!
//! // -----------------------------------------------------------------------------
//! // 		- Writer -
//! // 		Write "bye" and drop the connection
//! // -----------------------------------------------------------------------------
//! struct Writer;
//!
//! impl Reactor for Writer {
//!     type Input = TcpStream;
//!     type Output = ();
//!
//!     fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
//!         use Reaction::*;
//!         match reaction {
//!             Value(mut stream) => {
//!                 stream.write(b"bye\n");
//!                 Continue
//!             }
//!             Event(ev) => Continue,
//!             Continue => Continue,
//!         }
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     System::init()?;
//!
//!     // Listen for incoming connections
//!     let listener = ReactiveTcpListener::bind("127.0.0.1:8000")?.map(|(s, _)| s);
//!     // Connection queue for connections to be sent to another thread.
//!     let mut queue = ReactiveQueue::unbounded();
//!
//!     for _ in 0..4 {
//!         let deque = queue.deque();
//!         thread::spawn(move || -> Result<()> {
//!             System::init()?;
//!             let deque = ReactiveDeque::new(deque)?;
//!             let writer = Writer;
//!             let run = deque.chain(writer);
//!             System::start(run)?;
//!             Ok(())
//!         });
//!     }
//!
//!     let run = listener.chain(queue);
//!     System::start(run)?;
//!     Ok(())
//! }
//! ```
//!
//!
use mio::{Event, Evented, Ready, Token};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, ErrorKind::WouldBlock, Read, Write};
use std::marker::PhantomData;

use super::system::System;
use crate::errors::Result;

mod combinators;
pub mod consumers;
pub mod producers;

pub use combinators::{And, Chain, Either, Map, Or};

/// Input / Output of a [`Reactor`].
///
/// [`Reactor`]: trait.Reactor.html
pub enum Reaction<T> {
    /// Continue
    Continue,

    /// A Mio event.
    Event(Event),

    /// Value
    Value(T),
}

impl<T> From<Event> for Reaction<T> {
    fn from(event: Event) -> Reaction<T> {
        Reaction::Event(event)
    }
}

impl<T: Debug> Debug for Reaction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Reaction::Continue => write!(f, "Reaction::Continue"),
            Reaction::Event(event) => write!(f, "Reaction::Event({:?})", event),
            Reaction::Value(val) => write!(f, "Reaction::Value({:?})", val),
        }
    }
}

/// A reactor reacts to a [`Reaction`] and returns a [`Reaction`].
///
/// With the `Output` type of one reactor being the same type as `Input` of another it's possible to chain these
/// two reactors together.
///
/// [`Reaction`]: enum.Reaction.html
pub trait Reactor: Sized {
    /// The output passed to the next reactor in the chain.
    type Output;

    /// Expected input type from the previous reactor in the chain.
    type Input;

    /// The generated output is passed as the input to the
    /// next reactor in the chain.
    ///
    /// `react` is called repeatedly until the reaction returns
    /// `Reaction::Continue`
    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output>;

    /// Chain two reactors together.
    /// The output of the first reactor is the input of the second reactor.
    fn chain<T: Reactor>(self, to: T) -> Chain<Self, T> {
        Chain::new(self, to)
    }

    /// Run two reactors independent of each other.
    /// ```no_run
    /// # use sonr::reactor::Reactor;
    /// # use sonr::errors::Result;
    /// use sonr::system::System;
    /// use sonr::net::tcp::ReactiveTcpListener;
    ///
    /// fn main() -> Result<()> {
    ///     System::init();
    ///     let first_listener = ReactiveTcpListener::bind("127.0.0.1:5000")?;
    ///     let second_listener = ReactiveTcpListener::bind("127.0.0.1:5001")?;
    ///     let server = first_listener.and(second_listener);
    ///     System::start(server);
    /// #   Ok(())
    /// }
    /// ```
    fn and<C: Reactor>(self, second: C) -> And<Self, C> {
        And::new(self, second)
    }

    /// Capture the output of a reactor in a closure.
    /// ```no_run
    /// // Create a listener, print the address every time
    /// // the listener accepts a new connection, then push that
    /// // connection onto a queue.
    /// # use sonr::net::tcp;
    /// # use sonr::sync::queue::ReactiveQueue;
    /// # use sonr::errors::Result;
    /// # use sonr::prelude::*;
    /// # fn main() -> Result<()> {
    /// System::init();
    /// let listener = tcp::ReactiveTcpListener::bind("127.0.0.1:5000")?;
    /// let queue = ReactiveQueue::unbounded();
    /// let server = listener.map(|(stream, addr)| {
    ///     stream
    /// }).chain(queue);
    /// System::start(server);
    /// # Ok(())
    /// # }
    /// ```
    fn map<F, T>(self, callback: F) -> Map<Self, F, T> {
        Map::new(self, callback)
    }

    /// Pass the output from a reactor into one of two
    /// reactors depending on the output.
    /// Note that both `Reactor`s in an `or` are required
    /// to have the same output, and it's only possible to `chain`
    /// two `or`ed reactors if the reactor that does the chaining outputs
    /// `Either`.
    ///
    ///```
    /// # use sonr::prelude::*;
    /// # use sonr::errors::Result;
    /// use sonr::reactor::consumers::Consume;
    /// use sonr::reactor::producers::Mono;
    /// use sonr::reactor::Either;
    ///
    /// fn main() -> Result<()> {
    ///     let system_sig = System::init()?;
    ///     let producer = Mono::new(1u32)?
    ///         .map(|val| {
    ///             if val == 1 {
    ///                 Either::A(val)
    ///             } else {
    ///                 Either::B(val)
    ///             }
    ///         });
    ///
    ///     let reactor_a = Consume::new();
    ///     let reactor_b = Consume::new();
    ///     let reactor = reactor_a.or(reactor_b)
    ///         .map(|_| {
    ///             system_sig.send(SystemEvent::Stop);
    ///         });
    ///
    ///     let run = producer.chain(reactor);
    ///
    ///     System::start(run)?;
    ///     Ok(())
    /// }
    /// ```
    fn or<T: Reactor>(self, second: T) -> Or<Self, T> {
        Or::new(self, second)
    }
}

// -----------------------------------------------------------------------------
// 		- An evented Reactor -
// -----------------------------------------------------------------------------
/// The `EventedReactor` is driven by the [`System`].
///
/// An `EventedReactor` can not be sent between threads as it's bound to the
/// System it was registered with.
///
/// When an `EventedReactor` is created it's automatically registered with the [`System`],
/// and when it dropps the [`Token`] registered with the `EventedReactor` is freed
/// to be reused with another `EventedReactor`.
///
/// The `EventedReactor` does not implement [`Reactor`] by it self,
/// but rather acts as a building block when creating a Reactor that should
/// also be evented.
///
/// The [`Stream`] is an example of this.
///
/// [`Reactor`]: trait.Reactor.html
/// [`Stream`]: ../net/stream/struct.Stream.html
/// [`System`]: ../system/struct.System.html
/// [`Token`]: ../struct.Token.html
pub struct EventedReactor<E: Evented> {
    inner: E,
    token: Token,
    interest: Ready,
    pub(crate) is_readable: bool,
    pub(crate) is_writable: bool,
    _not_send: PhantomData<*const ()>, // Make the evented reactor !Send
}

impl<E> Debug for EventedReactor<E>
where
    E: Debug + Evented,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("EventedReactor")
            .field("inner", &self.inner)
            .field("token", &self.token)
            .field("interest", &self.interest)
            .field("is_readable", &self.is_readable)
            .field("is_writable", &self.is_writable)
            .finish()
    }
}

impl<E: Evented> EventedReactor<E> {
    /// Create a new instance of an `EventedReactor`.
    pub fn new(inner: E, interest: Ready) -> Result<Self> {
        let token = System::reserve_token()?;
        System::register(&inner, interest, token)?;

        Ok(Self {
            inner,
            token,
            interest,
            is_readable: false,
            is_writable: false,
            _not_send: PhantomData,
        })
    }

    /// Reference to the underlying evented type
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Mutable reference to the underlying evented type
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Return the `Token` used to register the inner type with
    /// poll.
    pub fn token(&self) -> Token {
        self.token
    }

    /// Return the interests of the reactor, usually readable and/or writable.
    pub fn interest(&self) -> Ready {
        self.interest
    }
}

impl<E: Evented + Read> Read for EventedReactor<E> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.inner_mut().read(buf);

        match res {
            Err(ref e) if e.kind() == WouldBlock => {
                self.is_readable = false;
                let res = System::reregister(&self);
                match res {
                    Ok(()) => (),
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to reregister evented: {:?}", e),
                        ));
                    }
                }
            }
            Err(_) => self.is_readable = false,
            Ok(0) => self.is_readable = false,
            _ => {}
        }

        res
    }
}

impl<E: Evented + Write> Write for EventedReactor<E> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner_mut().write(buf);

        match res {
            Err(ref e) if e.kind() == WouldBlock => {
                self.is_writable = false;
                let res = System::reregister(&self);
                match res {
                    Ok(()) => (),
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to reregister evented: {:?}", e),
                        ));
                    }
                }
            }
            Err(_) => self.is_writable = false,
            _ => {}
        }

        res
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner_mut().flush()
    }
}

impl<E: Evented> Drop for EventedReactor<E> {
    fn drop(&mut self) {
        System::free_token(self.token());
    }
}
