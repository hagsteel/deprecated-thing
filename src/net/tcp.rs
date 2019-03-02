use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;

use mio::{Event, Ready, Token};

use crate::errors::Result;
use crate::net::stream::Stream;
use crate::reactor::Reactive;
use crate::reactor::{EventedReactor, Reaction};
use crate::system::System;

// Re-exports
pub use mio::net::{TcpStream, TcpListener};

// -----------------------------------------------------------------------------
//              - Tcp Listener -
// -----------------------------------------------------------------------------
pub struct ReactiveTcpListener {
    inner: EventedReactor<mio::net::TcpListener>,
}

impl ReactiveTcpListener {
    pub fn new(listener: mio::net::TcpListener) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(listener, Ready::readable())?,
        })
    }

    pub fn bind(addr: &str) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(
                mio::net::TcpListener::bind(&addr.parse()?)?,
                Ready::readable(),
            )?,
        })
    }

    pub fn token(&self) -> Token {
        self.inner.token()
    }
}

impl Reactive for ReactiveTcpListener {
    type Output = (mio::net::TcpStream, SocketAddr);
    type Input = ();

    fn reacting(&mut self, event: Event) -> bool {
        self.inner.token() == event.token()
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        let res = self.inner.inner().accept();
        match res {
            Ok(val) => return Reaction::Value(val),
            Err(ref e) if e.kind() == WouldBlock => {
                System::reregister(&self.inner).unwrap();
            }
            Err(_) => (),
        }
        Reaction::NoReaction
    } 
}

pub type ReactiveTcpStream = Stream<mio::net::TcpStream>;