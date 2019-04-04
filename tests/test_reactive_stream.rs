use std::collections::HashMap;
use sonr::net::{stream, tcp};
use sonr::reactor::{Reaction, Reactor};
use sonr::sync::signal::SignalSender;
use sonr::system::{System, SystemEvent};
use sonr::{Event, Token};
use std::io::{Read, Write};
use std::thread;

type WriteBuffer = Vec<u8>;

struct Connections {
    streams: HashMap<Token, (tcp::ReactiveTcpStream, WriteBuffer)>
}

impl Connections {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }
}

impl Reactor for Connections {
    type Input = tcp::ReactiveTcpStream;
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Value(stream) => {
                // New stream
                self.streams.insert(stream.token(), (stream, WriteBuffer::new()));
                Continue
            }
            Event(event) => {
                // Check if the event belongs to one of the streams, otherwise
                // pass the event to the next reactor
                if let Some((stream, write_buffer)) = self.streams.get_mut(&event.token()) {
                    stream.react(event.into());

                    // Read
                    while stream.readable() {
                        // Read until the stream block
                        // as the stream will not receive
                        // a new read event until it blocks
                        break
                    }

                    while stream.writable() && !write_buffer.is_empty() {
                        // Write to the stream until there is nothing 
                        // left in the write buffer
                        break
                    }

                    Continue
                } else {
                    event.into()
                }
            }
            Continue => Continue,
        }
    }
}

#[test]
fn test_tcp_stream() {
    let system_sig = System::init().unwrap();

    // let handle = thread::spawn(move || {
    //     System::init();
    //     let tcp = tcp::ReactiveTcpListener::bind("127.0.0.1:5555").unwrap();
    //     let tcp = tcp.map(|(mut stream, addr): (tcp::TcpStream, _)| {
    //         stream.write(&b"hi"[..]);
    //         (stream, addr)
    //     });
    //     System::start(tcp);
    // });

    // thread::sleep_ms(200);
    // let stream = tcp::TcpStream::connect(&"127.0.0.1:5555".parse().unwrap()).unwrap();
    // let client = TcpClient::new(tcp::ReactiveTcpStream::new(stream).unwrap(), system_sig);
    // System::start(client);
}
