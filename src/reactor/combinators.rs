//! Combine [`Reactor`]s creating new [`Reactor`]s.
//!
//! See each individual combinator for more information.
use std::marker::PhantomData;

use super::{Reaction, Reactor};

/// Chain two [`Reactor`]s together, making the output of the first
/// reactor the input of the second.
///
/// [`Reactor`]: ../trait.Reactor.html
pub struct Chain<F, T>
where
    F: Reactor,
    T: Reactor,
{
    from: F,
    to: T,
}

impl<F, T> Chain<F, T>
where
    F: Reactor,
    T: Reactor,
{
    /// Create a chain of two [`Reactors`].
    ///
    /// [`Reactor`]: ../trait.Reactor.html
    pub fn new(from: F, to: T) -> Self {
        Self { from, to }
    }
}

impl<F, T> Reactor for Chain<F, T>
where
    F: Reactor,
    T: Reactor<Input = F::Output>,
{
    type Input = F::Input;
    type Output = T::Output;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        let mut r1 = self.from.react(reaction);
        loop {
            match r1 {
                Reaction::Event(_) => break self.to.react(r1),
                Reaction::Value(val) => {
                    let _ = self.to.react(Reaction::Value(val));
                    r1 = self.from.react(Reaction::Continue);
                }
                Reaction::Continue => {
                    if let Reaction::Continue = self.to.react(Reaction::Continue) {
                        break Reaction::Continue;
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 		- And two reactors -
// -----------------------------------------------------------------------------
/// Use `and` to run more than one (evented) reactor in parallel.
/// 
/// Since the output of a tcp listener is a stream and a socket address, it would
/// not be possible to chain two tcp listeners together.
/// It is also not possible to call `System::start` twice in the same thread.
/// 
/// To run two tcp listeners at the same time use `and`:
/// 
/// ```ignore
/// use sonr::prelude::*;
/// use sonr::net::tcp::ReactiveTcpListener;
/// 
/// fn main() {
///     System::init();
/// 
///     let listener_1 = ReactiveTcpListener::bind("127.0.0.1:8000").unwrap();
///     let listener_2 = ReactiveTcpListener::bind("127.0.0.1:9000").unwrap();
/// 
///     System::start(listener_1.and(listener_2));
/// }
/// ```
/// 
/// This means a `Reaction::Event(event)` from the `System` will be passed on to both
/// listeners.
pub struct And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    first: T,
    second: U,
}

impl<T, U> And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    /// Create a new `And` from two reactors.
    pub fn new(first: T, second: U) -> Self {
        Self { first, second }
    }
}

impl<T, U> Reactor for And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    type Output = ();
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(event) => {
                self.first.react(Reaction::Event(event));
                self.second.react(Reaction::Event(event));
                Reaction::Event(event)
            }
            _ => Reaction::Continue,
        }
    }
}

// -----------------------------------------------------------------------------
// 		- Map -
// -----------------------------------------------------------------------------
/// Map will capture the `Reaction::Value(val)` returned by `react` and apply 
/// the provided closure on the value.  
///
/// The returned value from the map is the new `Output` of the Reactor.
///
///```
/// # use std::time::Duration;
/// # use std::net::SocketAddr;
/// # use std::thread;
/// # use std::net::TcpStream as StdStream;
/// # use sonr::prelude::*;
/// # use sonr::errors::Result;
/// use sonr::net::tcp::{ReactiveTcpListener, TcpStream};
///
/// fn main() -> Result<()> {
///     let system_signals = System::init()?;
/// 
///     let listener = ReactiveTcpListener::bind("127.0.0.1:5555")?;
///     let run = listener.map(|(strm, addr)| {
///         // Return the tcp stream, ignoring the SocketAddr
///         strm
///     }); 
///     # thread::spawn(move || {
///     #     thread::sleep(Duration::from_millis(100));
///     #     StdStream::connect("127.0.0.1:5555");
///     #     system_signals.send(SystemEvent::Stop);
///     # });
/// 
///     System::start(run)?;
///     Ok(())
/// }
/// ```
pub struct Map<S, F, T> {
    source: S,
    callback: F,
    _p: PhantomData<T>,
}

impl<S, F, T> Map<S, F, T> {
    /// Create a new map from a reactor and a closure.
    pub fn new(source: S, callback: F) -> Self {
        Self {
            source,
            callback,
            _p: PhantomData,
        }
    }
}

impl<S, F, T> Reactor for Map<S, F, T>
where
    S: Reactor,
    F: FnMut(S::Output) -> T,
{
    type Output = T;
    type Input = S::Input;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        let reaction = self.source.react(reaction);
        match reaction {
            Reaction::Value(val) => Reaction::Value((self.callback)(val)),
            Reaction::Event(event) => Reaction::Event(event),
            Reaction::Continue => Reaction::Continue,
        }
    }
}
