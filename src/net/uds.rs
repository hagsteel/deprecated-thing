use std::io::ErrorKind::WouldBlock;
use std::os::unix::net::SocketAddr;

use mio::{Event, Ready, Token};

use crate::reactor::Reactive;
use crate::reactor::{Reaction, EventedReactor};
use crate::system::System;
use crate::errors::Result;
use crate::net::stream::Stream;

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

impl Reactive for ReactiveUdsListener {
    type Output = (UnixStream, SocketAddr);
    type Input = ();

    fn reacting(&mut self, event: Event) -> bool { //Reaction<Self::Output> {
        self.inner.token() == event.token()
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        let res = self.inner.inner().accept();
        match res {
            Ok(Some(val)) => return Reaction::Value(val),
            Ok(None) => { }
            Err(ref e) if e.kind() == WouldBlock => {
                System::reregister(&self.inner).unwrap();
            }
            Err(_) => (),
        }
        Reaction::NoReaction
    }
}


pub type ReactiveUdsStream = Stream<UnixStream>;
