//! Stream

use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Write};

use mio::{Evented, Ready, Token};

use crate::errors::Result;
use crate::reactor::Reactor;
use crate::reactor::{EventedReactor, Reaction};

/// Anything that has a stream
pub trait StreamRef {
    /// The Evented type for the Stream
    type Evented: Evented + Read + Write;

    /// Immutable reference to the stream
    fn stream_ref(&self) -> &Stream<Self::Evented>;
    
    /// Mutable reference to the stream
    fn stream_mut(&mut self) -> &mut Stream<Self::Evented>;
}

// -----------------------------------------------------------------------------
// 		- Stream -
// -----------------------------------------------------------------------------
/// When a [`Stream`] `react`s the inner evented reactor
/// is marked as either readable and / or writable depending on the [`Ready`] state of the
/// [`Event`].
///
/// It is likely that a Stream<T> is used as part of a larger `Reactor`, and the `react` 
/// method is called not by the `System` but rather the container of the `Stream`:
///
///```
/// # use std::collections::HashMap;
/// use sonr::prelude::*;
/// use sonr::net::tcp::ReactiveTcpStream;
///
/// type WriteBuffer = Vec<u8>;
/// 
/// struct Connections {
///     streams: HashMap<Token, (ReactiveTcpStream, WriteBuffer)>
/// }
/// 
/// impl Connections {
///     pub fn new() -> Self {
///         Self {
///             streams: HashMap::new(),
///         }
///     }
/// }
/// 
/// impl Reactor for Connections {
///     type Input = ReactiveTcpStream;
///     type Output = ();
/// 
///     fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
///         use Reaction::*;
///         match reaction {
///             Value(stream) => {
///                 // New stream
///                 self.streams.insert(stream.token(), (stream, WriteBuffer::new()));
///                 Continue
///             }
///             Event(event) => {
///                 // Check if the event belongs to one of the streams, otherwise
///                 // pass the event to the next reactor
///                 if let Some((stream, write_buffer)) = self.streams.get_mut(&event.token()) {
///                     stream.react(event.into());
/// 
///                     // Read
///                     while stream.readable() {
///                         // Read until the stream block
///                         // as the stream will not receive
///                         // a new read event until it blocks
///                         break
///                     }
/// 
///                     while stream.writable() && !write_buffer.is_empty() {
///                         // Write to the stream until there is nothing 
///                         // left in the write buffer
///                         break
///                     }
/// 
///                     Continue
///                 } else {
///                     event.into()
///                 }
///             }
///             Continue => Continue,
///         }
///     }
/// }
/// # fn main() {
/// # }
///```
///
/// [`Stream`]: struct.Stream.html
/// [`Ready`]: ../../struct.Ready.html
/// [`Event`]: ../../struct.Event.html
pub struct Stream<T: Read + Write + Evented> {
    inner: EventedReactor<T>,
}

impl<T: Evented + Write + Read> AsRef<Stream<T>> for Stream<T> {
    fn as_ref(&self) -> &Stream<T> {
        &self
    }
}

impl<T> Stream<T>
where
    T: Debug + Evented + Read + Write,
{
    /// Consume the stream and return the underlying evented reactor
    pub fn into_inner(self) -> EventedReactor<T> {
        self.inner
    }
}

impl<T> Debug for Stream<T>
where
    T: Debug + Evented + Read + Write,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Stream")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Read + Write + Evented> From<EventedReactor<T>> for Stream<T> {
    fn from(reactor: EventedReactor<T>) -> Self {
        Self { inner: reactor }
    }
}

impl<T: Read + Write + Evented> Stream<T> {
    /// Create a new stream
    pub fn new(inner: T) -> Result<Self> {
        let inner = EventedReactor::new(inner, Ready::readable() | Ready::writable())?;
        Ok(Self { inner })
    }

    /// The token used to track readiness of the underlying stream
    pub fn token(&self) -> Token {
        self.inner.token()
    }

    /// Is the underlying object readable?
    pub fn readable(&self) -> bool {
        self.inner.is_readable
    }

    /// Is the underlying object writable?
    pub fn writable(&self) -> bool {
        self.inner.is_writable
    }

    /// Reference the underlying object
    pub fn inner(&self) -> &T {
        self.inner.inner()
    }

    /// Mutable reference to the underlying object
    pub fn inner_mut(&mut self) -> &mut T {
        self.inner.inner_mut()
    }
}

impl<T: Read + Write + Evented> Reactor for Stream<T> {
    type Output = ();
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(event) = reaction {
            if event.token() != self.inner.token() {
                return reaction;
            }

            self.inner.is_readable |= event.readiness().is_readable();
            self.inner.is_writable |= event.readiness().is_writable();

            Reaction::Value(())
        } else {
            reaction
        }
    }
}

impl<T: Read + Write + Evented> Read for Stream<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Write + Evented> Write for Stream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
