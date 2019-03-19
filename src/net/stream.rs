use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Write};

use mio::{Evented, Ready, Token};

use crate::errors::Result;
use crate::reactor::Reactor;
use crate::reactor::{EventedReactor, Reaction};

pub trait StreamRef {
    type Evented: Evented + Read + Write;

    fn stream_ref(&self) -> &Stream<Self::Evented>;
    fn stream_mut(&self) -> &mut Stream<Self::Evented>;
}

// -----------------------------------------------------------------------------
// 		- Stream -
// -----------------------------------------------------------------------------
/// When a [`Stream`] `react`s the inner evented reactor
/// is marked as either readable and / or writable.
///
/// TODO: document this
/// NOTE: Add a sensible example
///
/// [`Stream`]: struct.Stream.html
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
