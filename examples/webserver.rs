use std::io::Write;
use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::thread;

use sonr::{Token, Event};
use sonr::errors::Result;
use sonr::net::tcp::{ReactiveTcpListener, ReactiveTcpStream, TcpStream};
use sonr::reactor::{Reactor, Reaction};
use sonr::system::System;
use sonr::sync::queue::{ReactiveDeque, ReactiveQueue};

// -----------------------------------------------------------------------------
// 		- Disclaimer -
// 		This example will simply write the response to the
// 		underlying tcp stream until it either blocks
// 		or errors out. This is not how a web server should behave,
// 		but it's useful for testing.
// -----------------------------------------------------------------------------

static RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Length: 13\n\nHello World\n\n";

struct Connections {
    inner: HashMap<Token, ReactiveTcpStream>
}

impl Connections {
    fn handle_connection_event(&mut self, event: Event) -> bool {
        let reacting = self.inner.get(&event.token()).is_some();

        let stream = self.inner.remove(&event.token()).map(|mut stream| {
            stream.react(Reaction::Event(event));
            while stream.writable() {
                match stream.write(&RESPONSE) { 
                    Ok(_) => {},
                    Err(ref e) if e.kind() == WouldBlock => return Some(stream),
                    Err(e) => return None,
                }
            }
            return None
        });

        match stream {
            Some(Some(stream)) => { self.inner.insert(stream.token(), stream); }
            _ => {}
        }

        reacting
    }
}

impl Reactor for Connections {
    type Input = (TcpStream, SocketAddr);
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Value(input) => {
                let (stream, _) = input; // ignore address 
                let tcp_stream = ReactiveTcpStream::new(stream).unwrap(); 
                self.inner.insert(tcp_stream.token(), tcp_stream);
                Reaction::Continue
            }
            Reaction::Event(event) => {
                match self.handle_connection_event(event) {
                    true => Reaction::Continue,
                    false => Reaction::Event(event)
                }
            }
            Reaction::Continue => Reaction::Continue,
        } 
    }
}

fn main() -> Result<()> {
    System::init();
    let listener = ReactiveTcpListener::bind("127.0.0.1:5555")?;
    let mut stream_q = ReactiveQueue::unbounded();

    for _ in 0..8 {
        let deque = stream_q.deque();
        thread::spawn(move || {
            System::init();
            let incoming_streams = ReactiveDeque::new(deque).unwrap();
            let connections = Connections { inner: HashMap::new() };

            let streams = incoming_streams.chain(connections);
            System::start(streams);
        });
    }
        
    let server = listener.chain(stream_q);
    System::start(server);
    Ok(())
}
