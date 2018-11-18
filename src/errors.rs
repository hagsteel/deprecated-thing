//! SONR default `Error`
use std;
use mio::Token;

/// Result type: `std::result::Error<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;


/// Wrapping error type.
#[derive(Debug)]
pub enum Error {
    /// std::io::Error
    Io(std::io::Error),

    /// No connection:
    /// A connection with a specific `Token` no longer exists
    NoConnection(Token),

    /// The connection was removed either by closing
    /// the socket or through a socket error
    ConnectionRemoved(Token),

    /// The `PreVec` does not have capacity for the new entry
    NoCapacity,

    /// The session was already registered.
    /// A session can only be registered once
    /// (but reregistered multiple times)
    AlreadyRegistered
}


// -----------------------------------------------------------------------------
// 		- IO error -
// -----------------------------------------------------------------------------
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
