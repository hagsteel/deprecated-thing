//! Unix Domain Sockets

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
/// Accept incoming connections and output `(UnixStream, SocketAddr)`;
///
///```
/// # use std::time::Duration;
/// # use std::net::SocketAddr;
/// # use std::thread;
/// # use std::fs::remove_file;
/// # use sonr::prelude::*;
/// # use sonr::errors::Result;
/// use sonr::net::uds::{ReactiveUdsListener, UnixStream};
///
/// fn main() -> Result<()> {
///     let system_signals = System::init()?;
/// 
///     # remove_file("/tmp/sonr-uds-test.sock");
///     let listener = ReactiveUdsListener::bind("/tmp/sonr-uds-test.sock")?;
///     let run = listener.map(|(strm, addr)| {
///         eprintln!("connection from: {:?}", addr);
///         strm
///     }); 
///     # thread::spawn(move || {
///     #     thread::sleep(Duration::from_millis(100));
///     #     UnixStream::connect("/tmp/sonr-uds-test.sock");
///     #     system_signals.send(SystemEvent::Stop);
///     # });
/// 
///     System::start(run)?;
///     Ok(())
/// }
/// ```
pub struct ReactiveUdsListener {
    inner: EventedReactor<UnixListener>,
}

impl ReactiveUdsListener {
    /// Create a new Reactive UnixListener
    pub fn bind(path: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(
                UnixListener::bind(path.as_ref())?,
                Ready::readable(),
            )?,
        })
    }

    /// Get `Token` registered with the listener;
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


/// Type alias for `Stream<UnixStream>`
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
