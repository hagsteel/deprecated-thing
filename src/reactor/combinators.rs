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
// 		- Callback -
// -----------------------------------------------------------------------------
pub struct Callback<S, F> {
    source: S,
    callback: F,
}

impl<S, F> Callback<S, F> {
    pub fn new(source: S, callback: F) -> Self {
        Self { 
            source,
            callback,
        }
    }
}

impl<S, F> Reactive for Callback<S, F> 
    where
        S: Reactive,
        F: FnMut(S::Output) -> S::Output,
{
    type Output = S::Output;
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


// -----------------------------------------------------------------------------
// 		- Callback -
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
