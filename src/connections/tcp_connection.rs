pub use mio::net::TcpStream;
use mio::{Poll, PollOpt, Ready, Token};
use std::io;
use std::io::ErrorKind::{ConnectionReset, WouldBlock};
use std::io::{Read, Write, ErrorKind};
use std::net::{Shutdown, SocketAddr, ToSocketAddrs};

use super::Connection;

use errors::{Result, Error};

/// Tcp connection.
/// ```
/// # use std::time::Duration;
/// # use sonr::{Poll, Ready, Token, Events};
/// # use sonr::errors::Result;
/// use std::io::{Read, Write};
/// use sonr::connections::{Connection, TcpConnection};
///
/// # fn main() -> Result<()> {
/// let poll = Poll::new()?;
/// let mut events = Events::with_capacity(1024);
/// let token = Token(0);
///
/// let mut connection = TcpConnection::connect("127.0.0.1", 5000)?;
/// connection.interest().insert(Ready::readable() | Ready::writable());
/// connection.register(&poll, token);
///
/// poll.poll(&mut events, Some(Duration::from_millis(1)));
///
/// for event in &events {
///     if event.readiness().is_writable() {
///         connection.write(&b"foo"[..]);
///     }
///
///     if event.readiness().is_readable() {
///         let mut buf = [0u8; 100];
///         if let Ok(n) = connection.read(&mut buf) {
///             println!("read: {:?}", &buf[..n]);
///         }
///     }
/// }
/// 
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct TcpConnection {
    stream: TcpStream,
    interest: Ready,
}

impl TcpConnection {
    /// Create a new `TcpConnection` from a `TcpStream`
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            interest: Ready::empty(),
        }
    }

    /// Tries to connect to an address
    pub fn connect_with<T: ToSocketAddrs>(address: T) -> Result<Self> {
        let addr = address.to_socket_addrs()?.next();
        let addr = addr.ok_or(Error::Io(ErrorKind::InvalidInput.into()));
        let stream = TcpStream::connect(&addr?)?;
        let connection = Self::new(stream);
        Ok(connection)
    }

    /// Tries to connect to an address & port
    pub fn connect(addr: &str, port: u16) -> Result<Self> {
        let address = format!("{}:{}", addr, port)
            .to_socket_addrs()?
            .next()
            .unwrap_or_else(|| panic!("Failed to resolve: {}", addr));
        let stream = TcpStream::connect(&address)?;
        let connection = Self::new(stream);
        Ok(connection)
    }

    /// Return the peer address
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.stream.peer_addr()?)
    }
}

// -----------------------------------------------------------------------------
//              - Connection impl -
// -----------------------------------------------------------------------------
impl Connection for TcpConnection {
    fn register<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()> {
        poll.register(
            &self.stream,
            token.into(),
            self.interest,
            PollOpt::edge() | PollOpt::oneshot(),
        )
    }

    fn reregister<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()> {
        poll.reregister(
            &self.stream,
            token.into(),
            self.interest,
            PollOpt::edge() | PollOpt::oneshot(),
        )
    }

    fn interest(&mut self) -> &mut Ready {
        &mut self.interest
    }
}

impl Read for TcpConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.stream.read(buf) {
            Ok(0) => {
                info!("Closing connection (no bytes read)");
                self.interest = Ready::empty();
                Ok(0)
            }
            Ok(byte_count) => {
                info!("read {} bytes", byte_count);
                Ok(byte_count)
            }
            Err(e) => match e.kind() {
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
            },
        }
    }
}

impl Write for TcpConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.stream.write(buf) {
            Ok(byte_count) => {
                if byte_count == buf.len() {
                    // Remove the write interest as the provided
                    // slice was written.
                    //
                    // At this point a new write interest should be registered
                    // for the next slice.
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
impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> Self {
        Self::new(stream)
    }
}

// -----------------------------------------------------------------------------
// 		- impl AsMut -
// -----------------------------------------------------------------------------
impl AsMut<TcpConnection> for TcpConnection {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
