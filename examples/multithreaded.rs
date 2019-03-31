use std::io::Write;
use std::thread;
use sonr::prelude::*;
use sonr::errors::Result;
use sonr::sync::queue::{ReactiveQueue, ReactiveDeque};
use sonr::net::tcp::{ReactiveTcpListener, TcpStream, ReactiveTcpStream};

// -----------------------------------------------------------------------------
// 		- Writer -
// 		Writer "bye" and drop the connection
// -----------------------------------------------------------------------------
struct Writer;

impl Reactor for Writer {
    type Input = TcpStream;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Value(mut stream) => {
                stream.write(b"bye\n");
                Continue
            }
            Event(ev) => Continue,
            Continue => Continue,
        }
    }
}

fn main() -> Result<()> {
    System::init()?;

    // Listen for incoming connections
    let listener = ReactiveTcpListener::bind("127.0.0.1:8000")?.map(|(s, _)| s);
    // Connection queue for connections to be sent to another thread.
    let mut queue = ReactiveQueue::unbounded();

    for _ in 0..4 {
        let deque = queue.deque();
        thread::spawn(move || -> Result<()> {
            System::init()?;
            let deque = ReactiveDeque::new(deque)?;
            let writer = Writer;
            let run = deque.chain(writer);
            System::start(run)?;
            Ok(())
        });
    }

    let run = listener.chain(queue);
    System::start(run)?;
    Ok(())
}
