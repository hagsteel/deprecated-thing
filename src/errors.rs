//! SONR default `Error`
use std;
use std::net::AddrParseError;
use std::string::FromUtf8Error;
use mio::Token;

use crossbeam::channel::{TryRecvError, RecvError};

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
    AlreadyRegistered,

    /// No unix domain socket waiting for a connection
    NoUdsConnection,

    /// Try receive error
    TryRecvError(TryRecvError), 

    /// Receive error
    RecvError(RecvError), 

    /// Address parse error
    AddrParseError,

    /// From UTF8 error
    FromUtf8Error(FromUtf8Error),
}


// -----------------------------------------------------------------------------
// 		- IO error -
// -----------------------------------------------------------------------------
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

// -----------------------------------------------------------------------------
// 		- mpsc::RecvError -
// -----------------------------------------------------------------------------
impl From<RecvError> for Error {
    fn from(err: RecvError) -> Error {
        Error::RecvError(err)
    }
}

// -----------------------------------------------------------------------------
// 		- mpsc::TryRecvError -
// -----------------------------------------------------------------------------
impl From<TryRecvError> for Error {
    fn from(err: TryRecvError) -> Error {
        Error::TryRecvError(err)
    }
}

// -----------------------------------------------------------------------------
// 		- AddrParseError -
// -----------------------------------------------------------------------------
impl From<AddrParseError> for Error {
    fn from(_: AddrParseError) -> Error {
        Error::AddrParseError
    }
}

// -----------------------------------------------------------------------------
// 		- FromUtf8Error -
// -----------------------------------------------------------------------------
impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}

