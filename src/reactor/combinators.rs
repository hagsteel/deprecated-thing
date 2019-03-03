use std::marker::PhantomData;

use mio::Event;

use super::{Reactive, Reaction};

pub struct Chain<F, T>
    where
        F: Reactive,
        T: Reactive,
{
    from: F,
    to: T,
}

impl<F, T> Chain<F, T> 
    where
        F: Reactive,
        T: Reactive,
{ 
    pub fn new(from: F, to: T) -> Self {
        Self { 
            from,
            to,
        }
    }
}

impl<F, T> Reactive for Chain<F, T> 
    where
        F: Reactive,
        T: Reactive<Input=F::Output>,
{
    type Output = T::Output;
    type Input = F::Input;

    fn reacting(&mut self, event: Event) -> bool {
        if self.from.reacting(event) {
            while let Reaction::Value(val) = self.from.react() {
                self.to.react_to(val);
            }
        } else if self.to.reacting(event) {
            // This can not output anything
            // as it has no recipient, however
            // it's important that it reacts, or the 
            // last link in the chain won't ever execute
            // any code within it's `react` function.
            while let Reaction::Value(_) = self.from.react() { }
        }
        false
    }

    fn react_to(&mut self, input: Self::Input) {
        self.from.react_to(input)
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        self.to.react()
    }

}


// -----------------------------------------------------------------------------
// 		- And two reactors -
// -----------------------------------------------------------------------------
pub struct And<T, U>
    where
        T: Reactive,
        U: Reactive,
{
    first: T,
    second: U,
}

impl<T, U> And<T, U> 
    where
        T: Reactive,
        U: Reactive,
{

    pub fn new(first: T, second: U) -> Self {
        Self { 
            first,
            second,
        }
    }
}

impl<T, U> Reactive for And<T, U> 
    where
        T: Reactive,
        U: Reactive,
{
    type Output = ();
    type Input = ();

    fn reacting(&mut self, event: Event) -> bool {
        self.first.reacting(event);
        self.second.reacting(event);
        false
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        Reaction::Value(())
    }

    fn react_to(&mut self, _input: Self::Input) {
        unreachable!();
    }
}


// -----------------------------------------------------------------------------
// 		- Noop -
// -----------------------------------------------------------------------------
/// Useful as the final reactor in a chain.
/// Given a chain with two reactors the last reactor will never
/// have `react` invoked as there is no receiving reactor to accept
/// the output. Most cases this is not a problem unless the final
/// reactor is using `.map`.
///
/// This also makes it possible to run a single reactor.
///
/// ```
/// # use sonr::reactor::producers::ReactiveGenerator;
/// # use sonr::prelude::*;
/// # use sonr::errors::Result;
/// # fn main() -> Result<()> {
/// let handle = System::init()?;
/// let numbers = ReactiveGenerator::new(vec![1, 2, 3])?
///     .map(|number: usize| {
///         // This closure is never called unless noop is called.
///         eprintln!("{:?}", number * 2);
///         handle.send(SystemEvent::Stop);
///     });
///
/// // `numbers` has no receiving reactor for 
/// // the output, hence the closure in `map` is never invoked.
/// // However by adding a `noop()` call, numbers now has a recipient for the 
/// // output and the closure in map will be invoked
/// let run = numbers.noop();
///
/// System::start(run);
/// # Ok(())
/// # }
/// ```
pub struct Noop<T> {
    _p: PhantomData<T>,
}

impl<T> Noop<T> {
    pub fn new() -> Self {
        Self { 
            _p: PhantomData,
        }
    }
}

impl<T> Reactive for Noop<T> {
    type Output = ();
    type Input = T;

    fn reacting(&mut self, event: Event) -> bool {
        false
    }
}


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

impl<S, F, T> Reactive for Map<S, F, T> 
    where
        S: Reactive,
        F: FnMut(S::Output) -> T,
{
    type Output = T;
    type Input = S::Input;

    fn reacting(&mut self, event: Event) -> bool {
        self.source.reacting(event)
    }

    fn react_to(&mut self, input: Self::Input) {
        self.source.react_to(input);
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        match self.source.react() {
            Reaction::Value(val) => Reaction::Value((self.callback)(val)),
            Reaction::NoReaction => Reaction::NoReaction
        }
    }
}
