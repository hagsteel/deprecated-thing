use std::io::{Write, Read};
use std::thread;
use sonr::net::{tcp, stream};
use sonr::system::{System, SystemEvent};
use sonr::reactor::{Reactive, Reaction};
use sonr::sync::signal::SignalSender;
use sonr::Event;

// -----------------------------------------------------------------------------
// 		- Test tcp client -
// 		Reads "hi" from the underlying stream 
// 		and if successful it will issue a `Stop`
// 		system event and stop the main loop
// -----------------------------------------------------------------------------
struct TcpClient {
    stream: tcp::ReactiveTcpStream,
    system_sig: SignalSender<SystemEvent>
}

impl TcpClient {
    pub fn new(stream: tcp::ReactiveTcpStream, system_sig: SignalSender<SystemEvent>) -> Self {
        Self { 
            stream,
            system_sig,
        }
    }
}

impl Reactive for TcpClient {
    type Output = ();
    type Input = ();

    fn reacting(&mut self, event: Event) -> bool {
        let res = self.stream.reacting(event);

        if self.stream.readable() {
            let mut buf = [0u8; 2];
            self.stream.read(&mut buf);
            if &buf[..] == b"hi" {
                self.system_sig.send(SystemEvent::Stop);
            }
        }
        
        res
    }

    fn react(&mut self) -> Reaction<Self::Output> {
        Reaction::NoReaction
    }
}

#[test]
fn test_listener() {
    let system_sig = System::init().unwrap();

    let handle = thread::spawn(move || {
        System::init();
        let tcp = tcp::ReactiveTcpListener::bind("127.0.0.1:5555").unwrap();
        let tcp = tcp.and_then(|(mut stream, addr): (tcp::TcpStream, _)| { 
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
