use std::io::ErrorKind::WouldBlock;
use std::os::unix::net::SocketAddr;

use mio::{Ready, Token};

use crate::reactor::Reactor;
use crate::reactor::{Reaction, EventedReactor};
use crate::system::System;
use crate::errors::Result;
use crate::net::stream::{Stream, StreamRef};

// Re-exports
pub use mio_uds::{UnixListener, UnixStream};

// -----------------------------------------------------------------------------
//              - Uds Listener -
// -----------------------------------------------------------------------------
pub struct ReactiveUdsListener {
    inner: EventedReactor<UnixListener>,
}

impl ReactiveUdsListener {
    pub fn bind(path: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(
                UnixListener::bind(path.as_ref())?,
                Ready::readable(),
            )?,
        })
    }

    pub fn token(&self) -> Token {
        self.inner.token()
    }
}

impl Reactor for ReactiveUdsListener {
    type Output = (UnixStream, SocketAddr);
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(event) = reaction {
            if self.inner.token() == event.token() {
                let res = self.inner.inner().accept();
                match res {
                    Ok(Some(val)) => return Reaction::Value(val),
                    Ok(None) => return Reaction::Continue,
                    Err(ref e) if e.kind() == WouldBlock => {
                        System::reregister(&self.inner).unwrap();
                        return Reaction::Continue
                    }
                    Err(_) => return Reaction::Continue,
                }
            }
        }

        if let Reaction::Continue = reaction {
                let res = self.inner.inner().accept();
                match res {
                    Ok(Some(val)) => return Reaction::Value(val),
                    Ok(None) => return Reaction::Continue,
                    Err(ref e) if e.kind() == WouldBlock => {
                        System::reregister(&self.inner).unwrap();
                        return Reaction::Continue
                    }
                    Err(_) => return Reaction::Continue,
                }
        }

        match reaction {
            Reaction::Event(e) => Reaction::Event(e),
            Reaction::Continue => Reaction::Continue,
            Reaction::Value(_) => Reaction::Continue,
        } 
    } 
}


pub type ReactiveUdsStream = Stream<UnixStream>;

impl StreamRef for ReactiveUdsStream {
    type Evented = UnixStream;
    
    fn stream_ref(&self) -> &Self {
        self
    }

    fn stream_mut(&mut self) -> &mut Self {
        self
    }
}
