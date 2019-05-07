//! Consumers consume the output of other reactors.
use super::{Reaction, Reactor};
use std::marker::PhantomData;

/// Consume the output of a `Reactor`.
///
/// Mostly useful for testing.
/// ```
/// # use sonr::prelude::*;
/// # use sonr::reactor::producers::Mono;
/// # use sonr::errors::Result;
/// use sonr::reactor::consumers::Consume;
/// 
/// fn main() -> Result<()> {
///     let sys_sig = System::init()?;
/// 
///     let reactor_a = Mono::new(1u8)?;
///     let reactor_b = Consume::new().map(|_| sys_sig.send(SystemEvent::Stop));
/// 
///     System::start(reactor_a.chain(reactor_b));
///     # Ok(())
/// }
/// ```
pub struct Consume<T> {
    _p: PhantomData<T>,
}

impl<T> Consume<T> {
    /// Create a new `Consume`
    pub fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<T> Reactor for Consume<T> {
    type Input = T;
    type Output = T;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        reaction
    }
}
