#![deny(missing_docs)]
//! # Simple Oppinionated Networking in Rust
//!
//! Sonr is built on top of [mio](https://crates.io/crates/mio), with the 
//! goal of making networking in Rust a bit easier.
#[macro_use] extern crate log;
             extern crate mio;
#[cfg(unix)] extern crate mio_uds;
             extern crate net2;
             extern crate byteorder;

mod prevec;
pub mod server;
pub mod connections;
pub mod errors;

// Pub uses
pub use prevec::PreVec;

// Reexports
pub use mio::{Token, Ready, Event, Events, Poll, Evented};
