use std::io::Write;
use std::net::SocketAddr;
use sonr::prelude::*;
use sonr::net::tcp::{ReactiveTcpListener, TcpStream};
use sonr::errors::Result;

// -----------------------------------------------------------------------------
// 		- Writer -
// 		Write "bye" and drop the connection.
// -----------------------------------------------------------------------------
struct Writer;

impl Reactor for Writer {
    type Input = (TcpStream, SocketAddr);
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Value((mut stream, addr)) => {
                eprintln!("{:?} connected", addr);
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

    let listener = ReactiveTcpListener::bind("127.0.0.1:8000")?;
    let writer = Writer;
    let run = listener.chain(writer); 

    System::start(run)?;
    Ok(())
}
