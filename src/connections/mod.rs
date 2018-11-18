//! # Connections and Sessions
use std::ops::{Index, IndexMut};
mod tcp_connection;
mod uds_connection;
mod sessions;

pub use self::tcp_connection::TcpConnection;
pub use self::uds_connection::UdsConnection;
pub use self::sessions::{
    Sessions, 
    Session, 
    EventResult,
    EventReader,
    EventWriter,
};

use std::io;
use std::io::ErrorKind;
use std::io::{Read, Write};

use mio::{Poll, Token, Ready};

use prevec::PreVec;
use errors::*;

// -----------------------------------------------------------------------------
//              - Connection -
// -----------------------------------------------------------------------------
/// A trait representing a connection.
pub trait Connection : Read + Write {
    /// Register interest with the `poll` handle
    fn register<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()>;

    /// Reregister interest with the `poll` handle
    fn reregister<T: Into<Token>>(&self, poll: &Poll, token: T) -> io::Result<()>;

    /// Current interest (read and or write) assigned 
    /// to the connection
    fn interest(&mut self) -> &mut Ready;
}

// -----------------------------------------------------------------------------
//              - Connections -
// -----------------------------------------------------------------------------
/// A collection of connections.
/// It's recommended to use [`Sessions`] unless the functionality
/// is not covered by [`Sessions`].
///
/// [`Sessions`]: struct.Sessions.html
#[derive(Debug)]
pub struct Connections<T: Connection> {
    inner: PreVec<T>,
}

impl<T> Connections<T> 
    where 
        T: Connection,
{
    /// Creates a non-growable collection of `T: Connection`s.
    /// If more control is required than what is available in [`Sessions`]
    /// then use `Connections`.
    ///
    /// [`Sessions`]: struct.Sessions.html
    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = PreVec::with_capacity(capacity);
        inner.prevent_growth();
        Self { inner }
    }

    /// Insert and register a new connection.
    /// As with [`Sessions`] the read / write interest will be absent until
    /// the connection is reregistered with either read and / or write.
    pub fn insert_and_register(&mut self, poll: &Poll, connection: T) -> Result<(Token, &mut T)> {
        let index = self.inner.insert(connection)?;
        
        let token = Token(index);
        match self.inner.get_mut(token.into()) {
            Some(connection) => {
                connection.register(poll, token)?;
                Ok((token, connection))
            }
            None => Err(Error::NoConnection(token))
        }
    }

    /// Reregister the interest of the connection.
    ///
    /// ```
    /// # use sonr::connections::{Connections, Connection, UdsConnection};
    /// # use sonr::{Ready, Poll, Token};
    /// let poll = Poll::new().unwrap();
    /// let mut connections = Connections::<UdsConnection>::with_capacity(10);
    /// connections.get_mut(Token(1))
    ///     .map(|connection| {
    ///         *connection.interest() = Ready::writable() 
    ///                                | Ready::readable();
    ///     });
    /// connections.reregister(&poll, Token(1));
    /// ```
    pub fn reregister(&mut self, poll: &Poll, token: Token) -> Result<()> {
        match self.get_mut(token) {
            Some(conn) => conn.reregister(poll, token).or_else(|e| Err(e.into())),
            None => Err(Error::NoConnection(token))
        }
    }

    /// Get a reference to a [`T: Connection`]
    pub fn get<U: Into<usize>>(&self, token: U) -> Option<&T> {
        self.inner.get(token.into())
    }

    /// Get a mutable reference to a [`T: Connection`]
    pub fn get_mut<U: Into<usize>>(&mut self, token: U) -> Option<&mut T> {
        self.inner.get_mut(token.into())
    }

    /// Number of connections
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether there are connections or not
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove and disconnect a [`T: Connection`]
    fn remove<U: Into<usize> + Copy>(&mut self, token: U) {
        self.inner.remove(token.into());
    }

    /// Read from a connection into a buffer (`&[u8]`) returning number of
    /// bytes read if successful.
    ///
    /// If the read fails on any other error than `std::io::ErrorKind::WouldBlock`
    /// the connection is removed.
    ///
    /// The connection is also removed in the event of zero bytes read.
    ///
    /// `read` should be called until it blocks
    ///
    /// ```
    /// # use sonr::connections::{Connections, Connection, UdsConnection};
    /// # use sonr::{Event, Ready, Token, Poll};
    /// # fn poll_event() -> Event { Event::new(Ready::empty(), Token(1)) }
    /// use std::io::ErrorKind;
    /// use sonr::errors::Error::Io;
    /// let poll = Poll::new().unwrap();
    /// let mut connections = Connections::<UdsConnection>::with_capacity(10);
    ///
    /// // ... omitting setting up server etc.
    ///
    /// let event = poll_event();
    /// if event.readiness().is_readable() {
    ///     let mut buf = [0u8;1024];
    ///
    ///     loop {
    ///         let res =  connections.read(&poll, event.token(), &mut buf[..]);
    ///         match res {
    ///             Ok(bytes_read) => { /* buf has some data */ } 
    ///             Err(Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {
    ///                 connections.get_mut(event.token())
    ///                     .map(|con| con.interest().insert(Ready::readable()));
    ///                 connections.reregister(&poll, event.token());
    ///                 break
    ///             }
    ///             Err(_) => break // Connection was dropped
    ///         }
    ///     }
    /// }
    /// ```
    pub fn read(&mut self, poll: &Poll, token: Token, buffer: &mut [u8]) -> Result<usize> {
        let result = match self.get_mut(token) {
            Some(readable) => readable.read(buffer),
            None => return Err(Error::NoConnection(token)),
        };
        
        match result {
            Ok(0) => {
                self.remove(token);
                Err(Error::ConnectionRemoved(token))
            }
            Ok(n) => Ok(n),
            Err(e) => {
                match e.kind() {
                    ErrorKind::ConnectionRefused |
                    ErrorKind::ConnectionReset |
                    ErrorKind::ConnectionAborted => {
                        self.remove(token);
                        Err(Error::ConnectionRemoved(token))
                    }
                    ErrorKind::WouldBlock => {
                        if let Some(c) = self.get_mut(token) {
                            c.interest().insert(Ready::readable());
                            c.reregister(poll, token)?;
                        }
                        Err(Error::Io(e))
                    }
                    ioe => Err(Error::Io(ioe.into()))
                }

            }
        }
    }

    /// Write to a connection returning number of bytes written if successful.
    ///
    /// If the write fails on any other error than `std::io::ErrorKind::WouldBlock`
    /// the connection is removed.
    ///
    /// Unlike `read`, writing zero bytes does not assume a dropped connection, 
    /// as it could be the case of trying to write zero bytes or no more space
    /// left in a buffer
    ///
    /// ```
    /// # use sonr::connections::{Connections, Connection, UdsConnection};
    /// # use sonr::{Event, Ready, Token, Poll};
    /// # fn poll_event() -> Event { Event::new(Ready::empty(), Token(1)) }
    /// use std::io::ErrorKind;
    /// use sonr::errors::Error::Io;
    /// let poll = Poll::new().unwrap();
    /// let mut connections = Connections::<UdsConnection>::with_capacity(10);
    ///
    /// // ... omitting setting up server etc.
    ///
    /// let event = poll_event();
    /// if event.readiness().is_writable() {
    ///     let buf = b"foo";
    ///     let mut total_bytes_written = 0;
    ///
    ///     loop {
    ///         let res =  connections.write(&poll, event.token(), &buf[..]);
    ///         match res {
    ///             Ok(bytes_written) => {
    ///                 total_bytes_written += bytes_written;
    ///                 // Check if all bytes were written
    ///                 if total_bytes_written == buf.len() {
    ///                     connections.get_mut(event.token())
    ///                         .map(|con| con.interest().insert(Ready::readable()));
    ///                     connections.reregister(&poll, event.token());
    ///                     break;
    ///                 }
    ///             } 
    ///             Err(Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {
    ///                 connections.get_mut(event.token())
    ///                     .map(|con| con.interest().insert(Ready::writable()));
    ///                 connections.reregister(&poll, event.token());
    ///                 break
    ///             }
    ///             Err(_) => break // Connection was dropped
    ///         }
    ///     }
    /// }
    /// ```
    pub fn write(&mut self, poll: &Poll, token: Token, buffer: &[u8]) -> Result<usize> {
        let result = match self.get_mut(token) {
            Some(writable) => writable.write(buffer),
            None => return Err(Error::NoConnection(token)),
        };

        match result {
            Ok(0) => {
                self.remove(token);
                Err(Error::ConnectionRemoved(token))
            }
            Ok(n) => Ok(n),
            Err(e) => {
                match e.kind() {
                    ErrorKind::ConnectionRefused |
                    ErrorKind::ConnectionReset |
                    ErrorKind::ConnectionAborted => {
                        self.remove(token);
                        Err(Error::ConnectionRemoved(token))
                    }
                    ErrorKind::WouldBlock => {
                        if let Some(c) = self.get_mut(token) {
                            c.interest().insert(Ready::readable());
                            c.reregister(poll, token)?;
                        }
                        Err(Error::Io(e))
                    }
                    io_err => Err(Error::Io(io_err.into()))
                }
            }
        }
    }
}


// -----------------------------------------------------------------------------
// 		- impl Index -
// -----------------------------------------------------------------------------
impl<C, T> Index<T> for Connections<C> 
    where
        C: Connection,
        T: Into<usize>,
{
    type Output = C;

    fn index(&self, index: T) -> &Self::Output {
        &self.inner[index.into()]
    }
}

// -----------------------------------------------------------------------------
// 		- impl IndexMut -
// -----------------------------------------------------------------------------
impl<C, T> IndexMut<T> for Connections<C> 
    where
        C: Connection,
        T: Into<usize>,
{
    fn index_mut(&mut self, index: T) -> &mut C {
        &mut self.inner[index.into()]
    }
} 
