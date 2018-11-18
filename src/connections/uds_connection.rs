use std::io;
use std::path::Path;
use std::io::ErrorKind::{ConnectionReset, WouldBlock};
use std::io::{Read, Write};
use std::net::Shutdown;
use mio::{Ready, Poll, PollOpt, Token};
use mio_uds::UnixStream;

use super::Connection;

use errors::Result;


/// Uds connection.
/// ```
/// # use std::time::Duration;
/// # use sonr::{Poll, Ready, Token, Events};
/// # use sonr::errors::Result;
/// use std::io::{Read, Write};
/// use sonr::connections::{Connection, UdsConnection};
///
/// # fn main() -> Result<()> {
/// let poll = Poll::new()?;
/// let mut events = Events::with_capacity(1024);
/// let token = Token(0);
///
/// let c = UdsConnection::connect("/tmp/foo.sock");
/// match c {
///     Ok(mut connection) => {
///         connection.interest().insert(Ready::readable() | Ready::writable());
///         connection.register(&poll, token);
///
///         poll.poll(&mut events, Some(Duration::from_millis(1)));
///
///         for event in &events {
///             if event.readiness().is_writable() {
///                 connection.write(&b"foo"[..]);
///             }
///
///             if event.readiness().is_readable() {
///                 let mut buf = [0u8; 100];
///                 if let Ok(n) = connection.read(&mut buf) {
///                     println!("read: {:?}", &buf[..n]);
///                 }
///             }
///         }
///     }
///     _ => { println!("failed to connect"); }
/// }   
/// 
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct UdsConnection {
    stream: UnixStream,
    interest: Ready,
}

impl UdsConnection {
    /// Create a new `UdsConnection` from a unix stream
    pub fn new(stream: UnixStream) -> UdsConnection {
        Self {
            stream,
            interest: Ready::readable(),
        }
    }

    /// Connect to a given path
    pub fn connect<P: AsRef<Path>>(path: P) -> Result<UdsConnection> {
        let stream = UnixStream::connect(path)?;
        let connection = Self::new(stream);
        Ok(connection)
    }
}


// -----------------------------------------------------------------------------
//              - Connection impl -
// -----------------------------------------------------------------------------
impl Connection for UdsConnection {
    fn register<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()> {
        poll.register(
            &self.stream,
            token.into(),
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    fn reregister<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()> {
        poll.reregister(
            &self.stream,
            token.into(),
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        )
    }

    fn interest(&mut self) -> &mut Ready {
        &mut self.interest
    }
}

impl Read for UdsConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
         match self.stream.read(buf) {
            Ok(0) => { 
                info!("Closing connection (no bytes read)");
                self.interest = Ready::empty();
                Ok(0)
            }
            Ok(byte_count) => { 
                self.interest.insert(Ready::readable());
                Ok(byte_count)
            }
            Err(e) => {
                match e.kind() {
                    WouldBlock => { 
                        info!("Would block on read");
                        self.interest.insert(Ready::readable());
                        Err(e)
                    }
                    ConnectionReset => {
                        debug!("Connection reset by peer");
                        self.interest = Ready::empty();
                        Err(e)
                    }
                    _ => {
                        error!("Failed to read: {:?}", e);
                        self.interest = Ready::empty();
                        Err(e)
                    }
                }

            }
        }
    }
}

impl Write for UdsConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.stream.write(buf) {
            Ok(byte_count) => {
                self.interest.insert(Ready::readable());
                if byte_count == buf.len() {
                    self.interest.remove(Ready::writable());
                }
                info!("Bytes written: {}", byte_count);
                Ok(byte_count)
            }
            Err(e) => {
                match e.kind() {
                    WouldBlock => { 
                        info!("Would block on write");
                        Err(e)
                    }
                    ConnectionReset => {
                        debug!("Connection reset by peer");
                        self.interest = Ready::empty();
                        Err(e)
                    }
                    _ => {
                        error!("Failed to write: {:?}", e);
                        self.interest = Ready::empty();
                        Err(e)
                    }
                }

            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

// -----------------------------------------------------------------------------
// 		- impl From<Listener> -
// -----------------------------------------------------------------------------
impl From<UnixStream> for UdsConnection {
    fn from(stream: UnixStream) -> Self {
        Self::new(stream)
    }
}
