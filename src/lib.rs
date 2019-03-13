//! Simple Opinionated Networking in Rust
//!
//! Sonr builds on the idea of chaining reactors together to form the flow of the application.
//!
//!
//! ```text
//! TcpListener -> Authentication -> Chat
//! ```
//!
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
