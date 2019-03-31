//! Simple Opinionated Networking in Rust
//!
//! Sonr builds on the idea of chaining reactors together to form the flow of the application.
//!
//! The two main components of Sonr are the [`Reactor`] and the [`System`].
//!
//!
//! A [`Reactor`] reacts to the output of another reactor or a mio::[`Event`], and has
//! an input and an output. This makes it possible (and intended) to chain two
//! reactors. Such a chain is in it self a [`Reactor`], and can be chained further.
//! 
//! The [`System`] runs the reactors and handles the registration and re-registration
//! of reactors (using `mio::Poll::register`).
//!
//!```no_run
//! use std::io::Write;
//! use std::net::SocketAddr;
//! use sonr::prelude::*;
//! use sonr::net::tcp::{ReactiveTcpListener, TcpStream};
//! use sonr::errors::Result;
//! 
//! // -----------------------------------------------------------------------------
//! // 		- Writer -
//! // 		Write "bye" and drop the connection.
//! // -----------------------------------------------------------------------------
//! struct Writer;
//! 
//! impl Reactor for Writer {
//!     type Input = (TcpStream, SocketAddr);
//!     type Output = ();
//! 
//!     fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
//!         use Reaction::*;
//!         match reaction {
//!             Value((mut stream, addr)) => {
//!                 eprintln!("{:?} connected", addr);
//!                 stream.write(b"bye\n");
//!                 Continue
//!             }
//!             Event(ev) => Continue,
//!             Continue => Continue,
//!         }
//!     }
//! }
//! 
//! fn main() -> Result<()> {
//!     System::init()?;
//! 
//!     let listener = ReactiveTcpListener::bind("127.0.0.1:8000")?;
//!     let writer = Writer;
//!     let run = listener.chain(writer); 
//! 
//!     System::start(run)?;
//!     Ok(())
//! }
//! ```
//!
//! [`Reactor`]: reactor/trait.Reactor.html
//! [`System`]: system/struct.System.html
//! [`Event`]: struct.Event.html
#[deny(missing_docs)]
pub mod reactor;

#[deny(missing_docs)]
pub mod system;

#[deny(missing_docs)]
pub mod net;

#[deny(missing_docs)]
pub mod sync; 

#[deny(missing_docs)]
pub mod errors;

#[deny(missing_docs)]
mod prevec;

pub use prevec::PreVec;

// Re-exports
pub use mio::{Token, Event, Evented, PollOpt, Poll, Ready};

pub mod prelude {
    pub use mio::{Token, Event};
    pub use crate::reactor::{Reaction, Reactor};
    pub use crate::system::{SystemEvent, System};
    pub use crate::net::stream::Stream;
}
