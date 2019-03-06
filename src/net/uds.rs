use std::io::ErrorKind::WouldBlock;
use std::os::unix::net::SocketAddr;

use mio::{Ready, Token};

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

    // fn reacting(&mut self, event: Event) -> bool { //Reaction<Self::Output> {
    //     self.inner.token() == event.token()
    // }

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(event) = reaction {
            if self.inner.token() == event.token() {
                let res = self.inner.inner().accept();
                match res {
                    Ok(Some(val)) => return Reaction::Stream(val),
                    Ok(None) => return Reaction::NoReaction,
                    Err(ref e) if e.kind() == WouldBlock => {
                        System::reregister(&self.inner).unwrap();
                        return Reaction::NoReaction
                    }
                    Err(_) => return Reaction::NoReaction,
                }
            }
        }

        if let Reaction::NoReaction = reaction {
                let res = self.inner.inner().accept();
                match res {
                    Ok(Some(val)) => return Reaction::Stream(val),
                    Ok(None) => return Reaction::NoReaction,
                    Err(ref e) if e.kind() == WouldBlock => {
                        System::reregister(&self.inner).unwrap();
                        return Reaction::NoReaction
                    }
                    Err(_) => return Reaction::NoReaction,
                }
        }

        match reaction {
            Reaction::Event(e) => Reaction::Event(e),
            Reaction::NoReaction => Reaction::NoReaction,
            Reaction::Value(val) => Reaction::NoReaction,
            Reaction::Stream(val) => Reaction::NoReaction,
        } 
    } 
}


pub type ReactiveUdsStream = Stream<UnixStream>;
