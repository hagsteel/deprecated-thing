use sonr::net::{stream, tcp};
use sonr::reactor::{Reaction, Reactor};
use sonr::sync::signal::SignalSender;
use sonr::system::{System, SystemEvent};
use sonr::Event;
use std::io::{Read, Write};
use std::thread;

// -----------------------------------------------------------------------------
// 		- Test tcp client -
// 		Reads "hi" from the underlying stream
// 		and if successful it will issue a `Stop`
// 		system event and stop the main loop
// -----------------------------------------------------------------------------
struct TcpClient {
    stream: tcp::ReactiveTcpStream,
    system_sig: SignalSender<SystemEvent>,
}

impl TcpClient {
    pub fn new(stream: tcp::ReactiveTcpStream, system_sig: SignalSender<SystemEvent>) -> Self {
        eprintln!("{:?}", stream.token());
        Self { stream, system_sig }
    }
}

impl Reactor for TcpClient {
    type Output = ();
    type Input = ();

    // fn reacting(&mut self, event: Event) -> bool {
    //     let res = self.stream.reacting(event);

    //     if self.stream.readable() {
    //         let mut buf = [0u8; 2];
    //         self.stream.read(&mut buf);
    //         if &buf[..] == b"hi" {
    //             self.system_sig.send(SystemEvent::Stop);
    //         }
    //     }
    //
    //     res
    // }

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Event(event) = reaction {
            if let Reaction::Value(()) = self.stream.react(Reaction::Event(event)) {
                if self.stream.readable() {
                    let mut buf = [0u8; 2];
                    self.stream.read(&mut buf);
                    if &buf[..] == b"hi" {
                        self.system_sig.send(SystemEvent::Stop);
                    }
                }
            }

            // if !res {
            //     return Reaction::Event(event);
            // }
            //
            // if self.stream.readable() {
            //     let mut buf = [0u8; 2];
            //     self.stream.read(&mut buf);
            //     if &buf[..] == b"hi" {
            //         self.system_sig.send(SystemEvent::Stop);
            //     }
            // }
        }

        Reaction::Continue
    }
}

#[test]
fn test_listener() {
    let system_sig = System::init().unwrap();

    let handle = thread::spawn(move || {
        System::init();
        let tcp = tcp::ReactiveTcpListener::bind("127.0.0.1:5555").unwrap();
        let tcp = tcp.map(|(mut stream, addr): (tcp::TcpStream, _)| {
            stream.write(&b"hi"[..]);
            (stream, addr)
        });
        System::start(tcp);
    });

    thread::sleep_ms(200);
    let stream = tcp::TcpStream::connect(&"127.0.0.1:5555".parse().unwrap()).unwrap();
    let client = TcpClient::new(tcp::ReactiveTcpStream::new(stream).unwrap(), system_sig);
    System::start(client);
}
