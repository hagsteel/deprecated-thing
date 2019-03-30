//! Simple Opinionated Networking in Rust
//!
//! Sonr builds on the idea of chaining reactors together to form the flow of the application.
//!
//! The two main components of Sonr are the [`Reactor`] and the [`System`].
//!
//!
//! A [`Reactor`] is anything that reacts to the output of another reactor or a mio::[`Event`], and has
//! an input and an output. This makes it possible (and intended) to chain two
//! reactors. Such a chain is in it self a [`Reactor`], and can be chained further.
//! 
//! The [`System`] runs the reactors and handles the registration and re-registration
//! of reactors (using `mio::Poll::register`).
//!
//! [`Reactor`]: reactor/trait.Reactor.html
//! [`System`]: system/struct.System.html
//! [`Event`]: struct.Event.html
//#[deny(missing_docs)]
pub mod reactor;

//#[deny(missing_docs)]
pub mod system;
pub mod net;
pub mod sync; 
pub mod errors;
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
