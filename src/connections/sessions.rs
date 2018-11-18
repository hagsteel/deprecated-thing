use std::io;
use std::io::ErrorKind;
use std::ops::{Index, IndexMut};
use mio::{Poll, Token, Event, Ready};

use connections::Connection;
use prevec::PreVec;
use errors::*;

// -----------------------------------------------------------------------------
// 		- Event Result -
// -----------------------------------------------------------------------------
/// A result of a `read_buf` or `write_buf` call on a session.
pub enum EventResult {
    /// Number of bytes that was successfully read or written
    Ok(usize),

    /// The connection can no longer write / read
    ConnectionRemoved,

    /// The event is blocking and should be retried.
    /// This will happen on the next attempt at writing or reading
    /// and no special action is required
    Retry,

    /// There was no write or read event.
    NoEvent,
}


// -----------------------------------------------------------------------------
// 		- Session -
// -----------------------------------------------------------------------------
/// Unlike a connection the session has an internal buffer.
///
/// A session is required to be registered with a poll handle or it 
/// won't listen to events.
/// ```
/// # use sonr::{Token, Poll, Events};
/// # use sonr::connections::{TcpConnection, Session, EventResult};
/// # use sonr::errors::Result;
/// # fn main() -> Result<()> {
/// let poll = Poll::new()?;
/// let mut events = Events::with_capacity(1024);
/// let connection = TcpConnection::connect("127.0.0.1", 5000)?;
/// let mut session = Session::new(connection);
/// let token = Token(1);
/// 
/// // The session will listen to **read** events
/// session.register_readable(&poll, token)?;
///
/// // The session will listen to **write** events as well
/// session.reregister_writable(&poll, token)?;
///
/// # Ok(()) }
/// ```
pub struct Session<C> {
    connection: C,
    buffer: Vec<u8>,
    buffer_index: usize,
    registered: bool,
}

impl<C: Connection> Session<C> {
    /// Create a new session from a connection
    pub fn new(connection: C) -> Self {
        Self::with_capacity(connection, 1024*8)
    }

    /// Create a new session with the capacity for the internal buffer
    pub fn with_capacity(connection: C, capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        unsafe { buffer.set_len(capacity) }

        Self { 
            connection,
            buffer,
            buffer_index: 0,
            registered: false,
        }
    }

    pub(crate) fn fill_buffer(&mut self) -> io::Result<usize> {
        let res = self.connection.read(&mut self.buffer[..]);
        if let Ok(n) = res {
            self.buffer_index = n;
        }
        res
    }

    /// Expose a slice of the internal buffer with data from the 
    /// last `fill_buffer` call.
    /// The buffer will be empty until `fill_buffer` is called
    pub fn slice(&self) -> &[u8] {
        &self.buffer[..self.buffer_index]
    }

    /// Write data into a slice.
    /// Call this on an event with `event.readiness().is_writable()` or the call will block
    /// If the `Session` is handled outside of [`Sessions`] then make sure that:
    /// *  The token is the same token as the session was registered with.
    /// *  The session doesn't try to read / write again in the case of a `WriteResult::ConnectionRemoved`
    ///
    /// When using a `Session` together with the [`Sessions`] collection, the collection
    /// will manage the removal of the session.
    ///
    /// [`Sessions`]: struct.Sessions.html
    pub fn write_buf(&mut self, poll: &Poll, token: Token, buf: &[u8]) -> EventResult {
        let res = self.connection.write(buf);
        match res {
            Ok(0) => EventResult::ConnectionRemoved,
            Ok(byte_count) => EventResult::Ok(byte_count),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => { 
                match self.connection.reregister(poll, token) {
                    Ok(()) => EventResult::Retry,
                    Err(_e) => EventResult::ConnectionRemoved
                }
            }
            Err(_) => EventResult::ConnectionRemoved
        }
    }

    /// Read data into the internal buffer.
    /// Call this on an event with `event.readiness().is_readable()`.
    /// Calling this when the `Session` is not registered as readable
    /// or the token doesn't match the token the `Session` was registered
    /// with, will block.
    pub fn read_buf(&mut self, poll: &Poll, token: Token) -> EventResult {
        let res = self.fill_buffer();
        match res {
            Ok(0) => EventResult::ConnectionRemoved,
            Ok(n) => EventResult::Ok(n),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => { 
                match self.connection.reregister(poll, token) {
                    Ok(()) => EventResult::Retry,
                    Err(_e) => EventResult::ConnectionRemoved
                }
            }
            Err(_) => EventResult::ConnectionRemoved
        }
    }
    
    /// Register the underlying connection as writable.
    /// Either `register_readable` or `register_writable` should be called,
    /// and only once.
    ///
    /// If the `Session` is added to a [`Sessions`] collection then all 
    /// registration calls should be managed through [`Sessions`] and never
    /// directly on the `Session` instance.
    ///
    /// See [`reregister_readable`] and [`reregister_writable`] on [`Sessions`]
    ///
    /// [`reregister_readable`]: struct.Sessions.html#method.reregister_readable
    /// [`reregister_writable`]: struct.Sessions.html#method.reregister_writable
    /// [`Sessions`]: struct.Sessions.html
    pub fn register_writable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        if self.registered == true {
            return Err(Error::AlreadyRegistered);
        }

        self.connection.interest().insert(Ready::writable());
        self.connection.register(poll, token)?;
        Ok(())
    }

    /// Register the underlying connection as readable.
    /// See [`register_writable`] as the same information applies
    ///
    /// [`register_writable`]: struct.Session.html#method.register_writable
    pub fn register_readable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        if self.registered == true {
            return Err(Error::AlreadyRegistered);
        }
        self.connection.interest().insert(Ready::readable());
        self.connection.register(poll, token)?;
        Ok(())
    }

    /// Reregister the underlying connection as writable.
    /// Unlike [`register_readable`] and [`register_writable`], `reregister`
    /// can be called multiple times.
    ///
    /// In fact once an event is exhausted either [`reregister_readable`] and / or
    /// [`reregister_writable`] should be called as the `Session` will no longer
    /// listen to an event once it's complete.
    ///
    /// [`register_readable`]: struct.Session.html#method.register_readable
    /// [`register_writable`]: struct.Session.html#method.register_writable
    pub fn reregister_writable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        self.connection.interest().insert(Ready::writable());
        self.connection.reregister(poll, token)?;
        Ok(())
    }

    /// Reregister the underlying connection as readable.
    /// See [`register_writable`] as same information applies, excpet 
    /// for reading rather than writing to the underlying connection.
    ///
    /// [`register_writable`]: struct.Session.html#method.register_writable
    pub fn reregister_readable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        self.connection.interest().insert(Ready::readable());
        self.connection.reregister(poll, token)?;
        Ok(())
    }
}



// -----------------------------------------------------------------------------
// 		- Sessions -
// 		Similar to connections except for the session object.
// 		The API is not a one to one with connections
// -----------------------------------------------------------------------------
/// `Sessions` handles multiple connections.
///
/// ```
/// # use std::time::Duration;
/// # use sonr::{Token, Poll, Events};
/// # use sonr::server::{Server, tcp_listener};
/// # use sonr::connections::{TcpConnection, Session, Sessions};
/// # use sonr::errors::Result;
/// # fn main() -> Result<()> {
/// let poll = Poll::new()?;
/// let mut events = Events::with_capacity(1024);
/// let mut sessions = Sessions::<TcpConnection>::with_capacity(1_000_000);
/// 
/// // Set the server token to be outside the session capacity
/// // to ensure the servers token is never the same as a `sessions`
/// let server_token = 1_000_000 + 1;
///
/// // Create a server instance...
/// let mut tcp_server = Server::new(tcp_listener("127.0.0.1", 5000)?, server_token, &poll);
/// // ... and listen for incoming connections
/// tcp_server.listen()?;
///
/// // listen for incoming events.
/// // this would normally happen in a `loop { ... }`
/// let timeout = Duration::from_millis(1); // instantly time out
/// poll.poll(&mut events, Some(timeout));
///
/// for event in &events {
///     if event.token() == tcp_server.token() {
///         let stream = tcp_server.accept()?;
///         let session = Session::new(stream.into());
///         sessions.add(&poll, session);
///     } else {
///         if event.readiness().is_readable() {
///             sessions.try_read(&poll, &event)
///             .and_then(|buf| {
///                 // `buf` contains data read from the 
///                 // underlying connection.
///                 // this will be called until the
///                 // connection would block (or fails)
///             })
///             .done(|byte_count| {
///                 // Done is called once and only once if
///                 // anything was read, with the total number
///                 // of bytes read
///
///                 // reregister the connection
///                 // to listen to `write` events
///                 sessions.reregister_writable(
///                     &poll,
///                     event.token()
///                 );
///             });
///         }
///
///         if event.readiness().is_writable() {
///             let buf = b"payload";
///             sessions.try_write(&poll, &event)
///                 .with(|| { &buf[..] }) // payload to write
///                 .done(|byte_count| { println!("wrote {} bytes", byte_count); });
///         }
///     }
/// }
///
/// // The session will listen to **read** events
///
/// # Ok(()) }
/// ```
///
/// [`Session`]: struct.Session.html
pub struct Sessions<C> {
    inner: PreVec<Session<C>>,
}

impl<C> Sessions<C>
    where 
        C: Connection,
{
    /// Create a `Session` with a max number of connections.
    /// `Sessions` are built on top of a [`PreVec`] which prevents
    /// growth by default. This is to ensure that two `Sessions` don't overlap.
    ///
    /// [`PreVec`]: ../struct.PreVec.html
    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = PreVec::with_capacity(capacity);
        inner.prevent_growth();
        Self { inner, }
    }

    /// Create a `Session` with a max number of connections and a connection offset.
    /// This is useful in the event of multiple `Sessions` objects being registered
    /// with the same `Poll` handle.
    ///
    /// ```
    /// # use sonr::connections::{Sessions, TcpConnection, UdsConnection};
    /// let max_con = 100_000;
    ///
    /// // Adding a `Session` to tcp_sessions would give it 
    /// // a token of 1
    /// let mut tcp_sessions = Sessions::<TcpConnection>::with_capacity(max_con);
    ///
    /// // Adding a `Session` to uds_sessions would give it
    /// // a token of 100_000
    /// let mut uds_sessions = Sessions::<UdsConnection>::with_capacity_and_offset(100, max_con);
    /// ```
    ///
    /// [`PreVec`]: ../struct.PreVec.html
    pub fn with_capacity_and_offset(capacity: usize, offset: usize) -> Self {
        let mut inner = PreVec::with_capacity_and_offset(capacity, offset);
        inner.prevent_growth();
        Self { inner, }
    }

    /// Add a `Session` and register it with the `poll` handle.
    /// The `Session` is neither listening for read or write events at this point.
    ///
    /// To receive any events for the session call either 
    /// [`register_readable`] and / or [`register_writable`].
    ///
    /// [`register_readable`]: struct.Sessions.html#method.reregister_readable
    /// [`register_writable`]: struct.Sessions.html#method.reregister_writable
    pub fn add(&mut self, poll: &Poll, session: Session<C>) -> Result<(Token, &mut Session<C>)> {
        let token = Token(self.inner.insert(session)?);
        let session = &mut self.inner[token.into()];
        session.connection.register(poll, token)?;
        session.registered = true;
        Ok((token, session))
    }

    /// Remove and subsequently close the `Session` and underlying connection.
    /// It is safe to call remove multiple times.
    pub fn remove<T: Into<usize>>(&mut self, token: T) {
        self.inner.remove(token.into());
    }

    /// Get a mutable reference to a `Session`
    pub fn get_mut<T: Into<usize>>(&mut self, token: T) -> Option<&mut Session<C>> {
        self.inner.get_mut(token.into())
    }

    /// Reregister the underlying connection as writable.
    /// Calling `reregister_writable` will reregister interest for write events with
    /// the `Poll` handle.
    ///
    /// Writing should not happen outside of a write event
    pub fn reregister_writable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        self.get_mut(token).map(|session| {
            session.reregister_writable(poll, token)
        }).ok_or(Error::NoConnection(token))??;

        Ok(())
    }

    /// Register the underlying connection as readable.
    /// Calling `reregister_readable` will reregister interest for read events with
    /// the `Poll` handle.
    ///
    /// Reading outside of a read event will block
    pub fn reregister_readable(&mut self, poll: &Poll, token: Token) -> Result<()> {
        self.get_mut(token).map(|session| {
            session.reregister_readable(poll, token)
        }).ok_or(Error::NoConnection(token))??;

        Ok(())
    }

    /// Try to read from the underlying connection matching the token of the event.
    /// It's safe to call `try_read` without checking whether the event is readable or not
    /// as this will be checked before reading.
    /// In the event of calling `try_read` on an event that is not readable,
    /// the [`EventReader`] will return `ReadResult::NoEvent`.
    ///
    /// `try_read` will return an [`EventReader`].
    /// The [`EventReader`] will continuosly call `and_then` with a slice (`&[u8]`) for as long as the 
    /// connection is producing data.
    ///
    /// The slice needs to be handled in each `and_then` call as it won't be preserved
    /// between calls.
    ///
    /// For more information about how to handle read events see [`EventReader`].
    ///
    /// [`EventReader`]: struct.EventReader.html
    pub fn try_read<'session>(&'session mut self, poll: &'session Poll, event: &'session Event) -> EventReader<'session, C> { //ReadResult {
        EventReader {
            sessions: self,
            event,
            poll,
        }
    }

    /// Try to write to the underlying connection matching the token of the event.
    /// Like `try_read` it's safe to call without checking if the event is writable.
    ///
    /// `try_write` will return an [`EventWriter`].
    /// The [`EventWriter`] will call `with` accepting a closure wich should return the buffer to
    /// write.
    ///
    /// For more information about how to handle write events see [`EventWriter`].
    /// 
    /// [`EventWriter`]: struct.EventWriter.html
    pub fn try_write<'session>(&'session mut self, poll: &'session Poll, event: &'session Event) -> EventWriter<'session, C> {
        EventWriter {
            sessions: self,
            event,
            poll,
        }
    }
}


// -----------------------------------------------------------------------------
// 		- Event reader -
// -----------------------------------------------------------------------------
/// Handle reading data from the underlying connection of a [`Session`].
///
/// `and_then` is called repeatedly until the underlying connection 
/// blocks. If the read(s) was successful the closure passed to `done` is called 
/// with the total number of bytes read.
///
/// ```
/// # use sonr::connections::{Sessions, TcpConnection};
/// # use sonr::{Event, Ready, Token, Poll};
/// # fn poll_event() -> Event { Event::new(Ready::empty(), Token(1)) }
/// let poll = Poll::new().unwrap();
/// let mut sessions = Sessions::<TcpConnection>::with_capacity(10);
///
/// // ... omitting code for setting up a listener and polling events
///
/// let event = poll_event();
/// sessions.try_read(&poll, &event)
///     .and_then(|buffer| {
///         println!("{}", String::from_utf8_lossy(buffer));
///     }).done(|bytecount| {
///         println!("read {} bytes", bytecount);
///     });
///
/// ```
///
/// [`Session`]: struct.Session.html
pub struct EventReader<'session, C: 'session> {
    sessions: &'session mut Sessions<C>,
    event: &'session Event,
    poll: &'session Poll,
}

impl<'session, C: Connection> EventReader<'session, C> {
    fn next(&mut self) -> EventResult {
        if !self.event.readiness().is_readable() {
            return EventResult::NoEvent;
        }

        let token = self.event.token();
        let res = self.sessions.inner[token.into()].read_buf(self.poll, token);
        if let EventResult::ConnectionRemoved = res {
            self.sessions.inner.remove(token.into());
        }
        res
    }

    /// Calls provided closure with a byte slice containing the
    /// data read from the socket.
    ///
    /// If the provided event (from the `EventReader` is not readable
    /// the finalising closure `done` won't be called.
    pub fn and_then<F: FnMut(&[u8])>(&mut self, mut f: F) -> Done {
        if !self.event.readiness().is_readable() {
            return Done::Incomplete;
        }

        let mut done = Done::Incomplete;
        let mut bytes_read = 0;
        while let EventResult::Ok(byte_count) = self.next() {
            bytes_read += byte_count;
            let buf = self.sessions.inner[self.event.token().into()].slice();
            f(buf);
            done = Done::BytesProcessed(bytes_read);
        }

        done
    }
}


// -----------------------------------------------------------------------------
// 		- Event writer -
// -----------------------------------------------------------------------------
/// Handle writing data to the underlying connection of a [`Session`].
///
/// `with` is called once to provide the buffer (`&[u8]`) to be written.
/// If the write(s) was successful the closure passed to `done` is called with the
/// total number of bytes written.
///
/// ```
/// # use sonr::connections::{Sessions, TcpConnection};
/// # use sonr::{Event, Ready, Token, Poll};
/// # fn poll_event() -> Event { Event::new(Ready::empty(), Token(1)) }
/// let poll = Poll::new().unwrap();
/// let mut sessions = Sessions::<TcpConnection>::with_capacity(10);
///
/// // ... omitting code for setting up a listener and polling events
///
/// let event = poll_event();
/// let message = b"foo";
/// sessions.try_write(&poll, &event)
///     .with(|| {
///         &message[..]
///     }).done(|bytecount| {
///         println!("wrote {} bytes", bytecount);
///         if bytecount == message.len() {
///             println!("wrote enture payload");
///         } else {
///             println!("failed to write: {:?}", &message[bytecount..]);
///         }
///     });
///
/// ```
///
/// [`Session`]: struct.Session.html
pub struct EventWriter<'session, C: 'session> {
    sessions: &'session mut Sessions<C>,
    event: &'session Event,
    poll: &'session Poll,
}

impl<'session, C: Connection> EventWriter<'session, C> {
    fn write(&mut self, buf: &[u8]) -> EventResult {
        let token = self.event.token();
        let res = self.sessions.inner[token.into()].write_buf(self.poll, token, buf);

        if let EventResult::ConnectionRemoved = res {
            self.sessions.inner.remove(token.into());
        }

        res
    }

    /// Provides a slice (`&[u8]`) to write to the underlying connection.
    ///
    /// Writes will be attempted until either the entire slice is written, 
    /// or the connection blocks.
    ///
    /// If the write was successful the closure passed to `done` is called
    /// with the total number of bytes written.
    pub fn with<'buf, F: FnMut() -> &'buf [u8]>(&mut self, mut f: F) -> Done {
        if !self.event.readiness().is_writable() {
            return Done::Incomplete;
        }

        let mut done = Done::Incomplete;
        let mut buf = f();
        let mut total_bytes_written = 0;
        while let EventResult::Ok(bytes_written) = self.write(buf) {
            total_bytes_written += bytes_written;
            done = Done::BytesProcessed(total_bytes_written);
            buf = &buf[bytes_written..];
            if buf.is_empty() {
                break; 
            }
        }

        done
    }
}

// -----------------------------------------------------------------------------
// 		- Done -
// -----------------------------------------------------------------------------
/// Tracks the state or read and write.
pub enum Done {
    BytesProcessed(usize),
    Incomplete
}


impl Done {
    /// Closure called once and only once if the write or read
    /// was successful.
    pub fn done<F: FnMut(usize)>(self, mut f: F) {
        match self {
            Done::BytesProcessed(count) => { f(count); }
            Done::Incomplete => {} 
        }
    }
}

// -----------------------------------------------------------------------------
// 		- impl Index -
// -----------------------------------------------------------------------------
impl<C, T> Index<T> for Sessions<C> 
    where 
        C: Connection,
        T: Into<usize>,
{
    type Output = Session<C>;

    fn index(&self, index: T) -> &Self::Output {
        &self.inner[index.into()]
    }
}

// -----------------------------------------------------------------------------
// 		- impl IndexMut -
// -----------------------------------------------------------------------------
impl<C, T> IndexMut<T> for Sessions<C> 
    where 
        C: Connection,
        T: Into<usize>,
{
    fn index_mut(&mut self, index: T) -> &mut Session<C> {
        &mut self.inner[index.into()]
    }
} 
