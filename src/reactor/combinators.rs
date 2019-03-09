use std::marker::PhantomData;

use super::{Reaction, Reactor};

pub struct Chain<F, T>
where
    F: Reactor,
    T: Reactor,
{
    from: F,
    to: T,
}

impl<F, T> Chain<F, T>
where
    F: Reactor,
    T: Reactor,
{
    pub fn new(from: F, to: T) -> Self {
        Self { from, to }
    }
}

impl<F, T> Reactor for Chain<F, T>
where
    F: Reactor,
    T: Reactor<Input = F::Output>,
{
    type Input = F::Input;
    type Output = T::Output;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        let mut r1 = self.from.react(reaction);
        loop {
            match r1 {
                Reaction::Event(_) => break self.to.react(r1),
                Reaction::Value(val) => {
                    let _ = self.to.react(Reaction::Value(val));
                    r1 = self.from.react(Reaction::Continue);
                }
                Reaction::Continue => {
                    if let Reaction::Continue = self.to.react(Reaction::Continue) {
                        break Reaction::Continue;
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 		- And two reactors -
// -----------------------------------------------------------------------------
pub struct And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    first: T,
    second: U,
}

impl<T, U> And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    pub fn new(first: T, second: U) -> Self {
        Self { first, second }
    }
}

impl<T, U> Reactor for And<T, U>
where
    T: Reactor,
    U: Reactor,
{
    type Output = ();
    type Input = ();

    // fn reacting(&mut self, event: Event) -> bool {
    //     self.first.reacting(event);
    //     self.second.reacting(event);
    //     false
    // }

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(event) => {
                self.first.react(Reaction::Event(event));
                self.second.react(Reaction::Event(event));
                Reaction::Event(event)
            }
            _ => Reaction::Continue,
        }
    }

    // fn react_to(&mut self, _input: Self::Input) {
    //     unreachable!();
    // }
}

// // -----------------------------------------------------------------------------
// // 		- Noop -
// // -----------------------------------------------------------------------------
// /// Useful as the final reactor in a chain.
// /// Given a chain with two reactors the last reactor will never
// /// have `react` invoked as there is no receiving reactor to accept
// /// the output. Most cases this is not a problem unless the final
// /// reactor is using `.map`.
// ///
// /// This also makes it possible to run a single reactor.
// ///
// /// ```
// /// # use sonr::reactor::producers::ReactiveGenerator;
// /// # use sonr::prelude::*;
// /// # use sonr::errors::Result;
// /// # fn main() -> Result<()> {
// /// let handle = System::init()?;
// /// let numbers = ReactiveGenerator::new(vec![1, 2, 3])?
// ///     .map(|number: usize| {
// ///         handle.send(SystemEvent::Stop);
// ///     });
// ///
// /// // `numbers` has no receiving reactor for
// /// // the output, hence the closure in `map` is never invoked.
// /// // However by adding a `noop()` call, numbers now has a recipient for the
// /// // output and the closure in map will be invoked
// /// let run = numbers.noop();
// ///
// /// System::start(run);
// /// # Ok(())
// /// # }
// /// ```
// pub struct Noop<T> {
//     _p: PhantomData<T>,
// }
// 
// impl<T> Noop<T> {
//     pub fn new() -> Self {
//         Self { _p: PhantomData }
//     }
// }
// 
// impl<T> Reactor for Noop<T> {
//     type Output = ();
//     type Input = T;
// 
//     fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
//         Reaction::Continue
//     }
// 
//     // fn reacting(&mut self, event: Event) -> bool {
//     //     false
//     // }
// }

// -----------------------------------------------------------------------------
// 		- Map -
// -----------------------------------------------------------------------------
pub struct Map<S, F, T> {
    source: S,
    callback: F,
    _p: PhantomData<T>,
}

impl<S, F, T> Map<S, F, T> {
    pub fn new(source: S, callback: F) -> Self {
        Self {
            source,
            callback,
            _p: PhantomData,
        }
    }
}

impl<S, F, T> Reactor for Map<S, F, T>
where
    S: Reactor,
    F: FnMut(S::Output) -> T,
{
    type Output = T;
    type Input = S::Input;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        let reaction = self.source.react(reaction);
        match reaction {
            Reaction::Value(val) => Reaction::Value((self.callback)(val)),
            Reaction::Event(event) => Reaction::Event(event),
            Reaction::Continue => Reaction::Continue,
        }
    }
}
