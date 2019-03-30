# Sonr

### Note: this is a work in progress and the api may change
### Note: currently not on crates.io, use:

```
[dependencies]
sonr = {git = "https://github.com/hagsteel/sonr" }

```

------

Simple Opinionated Networking in Rust

The goal of Sonr is to provide a light weight networking library that makes it
easy to get started writing network applications in Rust, and have a reasonably
low barrier to entry.


*  [API docs (incomplete)](https://hagsteel.github.io/sonr/)
*  [Examples](https://github.com/hagsteel/sonr/tree/master/examples)

------

Sonr is built on top of Mio with the aim to make it easier to create networking
applications.

The two main components of Sonr are the `Reactor` and the `System`.  

A `Reactor` is anything that reacts to the output of another reactor, and has
an input and an output. This makes it possible (and intended) to chain two
reactors. Such a chain is in it self a `Reactor`, and can be chained further.

The `System` runs the reactors and handles the registration and re-registration
of reactors (using `mio::Poll::register`).

`System` is thread local and has to exist for each thread using `Reactors`.
There can only be *one* instance of a `System` per thread, and this instance is created by
calling `System::init()`. Calling `System::init()` twice in the same thread will
panic.

```rust
fn main() -> Result<(), sonr::errors::Error> {
    System::init()?

    let reactor_1 = Reactor1::new()?;
    let reactor_2 = Reactor2::new()?;
    let reactor_3 = Reactor3::new()?;

    let run = reactor_1.chain(reactor_2.chain(reactor_3));

    System::start(run)?;
    Ok(())
}
```

## Reactor

The reactor trait consists of two types and a method:

```rust
trait Reactor {
    type Input;
    type Output;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output>;
}
```

To chain one reactor with another the `Output` of the first one needs to be of
the same type as the `Input` of the second one.

The following example shows the creation of a chain from two reactors.
Since none of the reactors are responding to `Event`s there is no need for the
`System` to be initialised.

```rust
use sonr::prelude::*;

struct VecToString;

impl Reactor for VecToString {
    type Input = Vec<u8>;
    type Output = String;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Value(bytes) => Value(String::from_utf8(bytes).unwrap()),
            Event(ev) => Event(ev),
            Continue => Continue,
        }
    }
}

struct UppercaseString;

impl Reactor for UppercaseString {
    type Input = String;
    type Output = String;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Value(mut s) => {
                s.make_ascii_uppercase();
                Value(s)
            }
            Event(ev) => Event(ev),
            Continue => Continue,
        }
    }
}

fn main() {
    let input = "hello world".to_owned().into_bytes();

    let r1 = VecToString;
    let r2 = UppercaseString;
    let mut chain = r1.chain(r2);
    chain.react(Reaction::Value(input));
}
```

Since reactors in a chain will always push data forward and never return
anything other than `Reaction::Continue` it is not possible to capture the
output from `chain.react(Raction::Value(input));` here.

To print the result in the above example it would be possible to create a third
reactor that prints the `Input`, however there is a lot of boilerplate to create
yet another reactor so in this case the `map` function of a reactor could be
used.

### Map

Updating the previous example to use `map` to print the output of
`UppercaseString`:

```rust
fn main() {
    let input = "hello world".to_owned().into_bytes();

    let r1 = VecToString;
    let r2 = UppercaseString.map(|the_string| println!("{}", the_string));
    let mut chain = r1.chain(r2);
    chain.react(Reaction::Value(input));
}
```

The `map` function takes a closure as an argument, where the argument for the
closure is the `Output` of the reactor.
Since the output of `UppercaseString` is a `String`, the `map` function is
called with a closure `FnMut(UppercaseString::Output) -> T`.

This is a convenient way to change the output of a reactor.

The `ReactiveTcpListener` outputs a tuple: `(mio::net::TcpStream, SocketAddr)`.
The map function could be useful here if the `SocketAddr` is not required:

```rust
use sonr::prelude::*;
use sonr::net::tcp::ReactiveTcpListener;

fn main() {
    System::init();

    let listener = ReactiveTcpListener::bind("127.0.0.1:8000")
        .unwrap()
        .map(|(stream, _socket_addr)| stream); // ignore the SocketAddr

    System::start(listener);
}
```

### And

The last method on a Reactor is `and`.
This is useful to run more than one (evented) reactor in parallel.

Since the output of a tcp listener is a stream and a socket address, it would
not be possible to chain two tcp listeners together.
It is also not possible to call `System::start` twice in the same thread.

To run two tcp listeners at the same time use `and`:

```rust
use sonr::prelude::*;
use sonr::net::tcp::ReactiveTcpListener;

fn main() {
    System::init();

    let listener_1 = ReactiveTcpListener::bind("127.0.0.1:8000").unwrap();
    let listener_2 = ReactiveTcpListener::bind("127.0.0.1:9000").unwrap();

    System::start(listener_1.and(listener_2));
}
```

This means a `Reaction::Event(event)` from the `System` will be sent to both
listeners.
