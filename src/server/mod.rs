//! Servers and listeners
mod listeners;

pub use self::listeners::Listener;
pub use self::listeners::{tcp_listener, uds_listener};
use std::io::ErrorKind;
use mio::{Poll, PollOpt, Ready, Token};
use errors::{Result, Error};

// -----------------------------------------------------------------------------
// 		- Server -
// -----------------------------------------------------------------------------
/// A server wraps a listener and accepts incoming connections.
///
/// # Example
///
/// ```
/// # use sonr::errors::Result;
/// use sonr::server::{tcp_listener, Server};
/// use sonr::connections::TcpConnection;
/// use sonr::{Events, Poll};
///
/// # fn main() -> Result<()> {
/// let poll = Poll::new()?;
/// // A collection of events, populated by poll.
/// let mut events = Events::with_capacity(1024);
///
/// // A tcp server built from a listener. 
/// // The only thing a server does is wait and accept
/// // connections.
/// let server_token = 1_000_000;
/// let mut tcp_server = Server::new(tcp_listener("127.0.0.1", 5000)?, server_token, &poll);
///
/// // Start listening for incoming connections
/// tcp_server.listen()?;
///
/// // Connect to the server
/// let connection = TcpConnection::connect("127.0.0.1", 5000)?;
///
/// loop {
///     poll.poll(&mut events, None)?;
///     for event in &events {
///         if event.token() == tcp_server.token() {
///             // Accept a new connection
///             let stream = tcp_server.accept()?;
///             return Ok(());
///         }
///     }
/// }
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Server<'a, L>
where 
    L: Listener,
{
    listener: L,
    poll: &'a Poll,
    server_token: Token,
}


impl<'a, L> Server<'a, L>
where
    L: Listener,
{
    /// Create a new server instance from a listener and a token.
    /// The token has to be unique to the `Poll` instance, as no two
    /// entities should be registered with the same token.
    pub fn new(listener: L, token: impl Into<Token>, poll: &'a Poll) -> Self {
        Self {
            listener,
            server_token: token.into(),
            poll,
        }
    }

    fn reregister(&mut self) -> Result<()> {
        self.poll.reregister(
            &self.listener,
            self.server_token,
            Ready::readable(),
            PollOpt::edge() | PollOpt::oneshot(),
        )?;
        Ok(())
    }

    /// Returns the token used to create the server and register with a `Poll` instance.
    pub fn token(&self) -> Token {
        self.server_token
    }

    /// Accept incoming connections
    pub fn accept(&mut self) -> Result<L::Stream> {
        let result = self.listener.accept();
        match result {
            Ok(stream) => { 
                match self.reregister() {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:#?}", e);
                    }
                }
                return Ok(stream);
            }
            Err(Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {
                self.reregister()?;
            }
            _ => {}
        }

        result
    }

    /// Listen for incoming connections. Should only be called once
    pub fn listen(&mut self) -> Result<()> {
        self.poll.register(
            &self.listener,
            self.server_token,
            Ready::readable(),
            PollOpt::edge() | PollOpt::oneshot(),
        )?;

        Ok(())
    }
}
