//! Reactive Tcp networking

use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;

use mio::{Ready, Token};

use crate::errors::Result;
use crate::net::stream::{Stream, StreamRef};
use crate::reactor::Reactor;
use crate::reactor::{EventedReactor, Reaction};
use crate::system::System;

// Re-exports
pub use mio::net::{TcpStream, TcpListener};

// -----------------------------------------------------------------------------
//              - Tcp Listener -
// -----------------------------------------------------------------------------
/// Accept incoming connections and output `(TcpStream, SocketAddr)`;
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
///         eprintln!("connection from: {:?}", addr);
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
pub struct ReactiveTcpListener {
    inner: EventedReactor<mio::net::TcpListener>,
}

impl ReactiveTcpListener {
    /// Create a new listener from a mio::TcpListener
    pub fn new(listener: mio::net::TcpListener) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(listener, Ready::readable())?,
        })
    }

    /// Create a new listener from an address
    pub fn bind(addr: &str) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(
                mio::net::TcpListener::bind(&addr.parse()?)?,
                Ready::readable(),
            )?,
        })
    }

    /// Get `Token` registered with the listener;
    pub fn token(&self) -> Token {
        self.inner.token()
    }
}

impl Reactor for ReactiveTcpListener {
    type Output = (mio::net::TcpStream, SocketAddr);
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(event) = reaction {
            if self.inner.token() == event.token() {
                let res = self.inner.inner().accept();
                match res {
                    Ok(val) => return Reaction::Value(val),
                    Err(ref e) if e.kind() == WouldBlock => {
                        let _ = System::reregister(&self.inner);
                        return Reaction::Continue
                    }
                    Err(_) => return Reaction::Continue,
                }
            } else {
                return Reaction::Event(event);
            }
        }

        if let Reaction::Continue = reaction {
            let res = self.inner.inner().accept();
            match res {
                Ok(val) => return Reaction::Value(val),
                Err(ref e) if e.kind() == WouldBlock => {
                    let _ = System::reregister(&self.inner);
                    return Reaction::Continue
                }
                Err(_) => return Reaction::Continue,
            }
        }

        match reaction {
            Reaction::Event(e) => Reaction::Event(e),
            _ => Reaction::Continue,
        } 
    } 
}

/// A reactive tcp stream.
pub type ReactiveTcpStream = Stream<mio::net::TcpStream>;

impl ReactiveTcpStream {
    /// Create a new reactive tcp stream from a &SocketAddr
    pub fn connect(addr: &SocketAddr) -> Result<Self> {
        let stream = mio::net::TcpStream::connect(addr)?;
        Ok(Self::new(stream)?)
    }
}

impl StreamRef for ReactiveTcpStream {
    type Evented = mio::net::TcpStream;
    
    fn stream_ref(&self) -> &Self {
        self
    }

    fn stream_mut(&mut self) -> &mut Self {
        self
    }
}
