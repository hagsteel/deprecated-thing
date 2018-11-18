extern crate sonr;

use std::collections::VecDeque;

use sonr::connections::{TcpConnection, Sessions, Session};
use sonr::errors::Result;
use sonr::server::{tcp_listener, Server};
use sonr::{Events, Poll};


fn main() -> Result<()> {
    let server_token = 1_000_000;
    // Poll: pulls events and registers read / write interest on the socket.
    let poll = Poll::new()?;
    // A collection of events, populated by poll.
    let mut events = Events::with_capacity(1024);

    // A tcp server from a listener.
    // The only thing a server does is wait and accept connections.
    let mut tcp_server = Server::new(tcp_listener("127.0.0.1", 5000)?, server_token, &poll);

    // Start listening for connections.
    tcp_server.listen()?;

    // Create a collection to holds up to 20,000 sessions.
    // The collection handles reads and writes (as seen below).
    let mut sessions = Sessions::<TcpConnection>::with_capacity(20_000);

    // A queue to store messages to write
    let mut write_queue: VecDeque<String> = VecDeque::new();

    loop {
        // Poll drives the application 
        poll.poll(&mut events, None).unwrap();

        // Each event raised by the system (most likely a socket event, e.g read / write)
        for event in &events {
            // If the event belongs to the server:
            if tcp_server.token() == event.token() {
                // Accept a stream (tcp stream in this case)
                let stream = tcp_server.accept()?;

                // Create a connection from the stream.
                let connection: TcpConnection = stream.into();

                // Create a session from the connection.
                // The session has an internal buffer (unlike a `T: Connection`)
                // In this case `with_capacity` sets the buffer capacity of the session
                let session = Session::with_capacity(connection, 512);

                // Add the session to a collection of sessions.
                let (token, _) = sessions.add(&poll, session)?;

                // Set the session to trigger on read events
                sessions.reregister_readable(&poll, token)?;

                continue
            }

            // Read:
            // Try to read data from the socket.
            // If the event is readable the socket is ready to be read from.
            // Hand a reference to `poll` and `event` to the `read_event`.
            // This is used to re-register read / write interest in case of an io::Error(WouldBlock)
            // error (which is not an actual error but rather an indication that the socket can no
            // longer read at this moment in time)
            //
            // `map` is called every time data is successfully read from the socket.
            // The socket needs to be read until WouldBlock occurrs.
            // In the event of an unrecoverable socket error the connection will close 
            // and the socket dropped.
            //
            // `done` happens once and only once if there was one or more successful reads 
            sessions.try_read(&poll, &event).and_then(|buf| {
                // Create a string from the byte slice
                let s =  String::from_utf8_lossy(buf);
                // Push the string onto a write queue.
                write_queue.push_back(s.to_string());
            }).done(|bytecount| { // until it's done
                println!("read {} bytes", bytecount);
                // Register interest for the socket to be 
                // interested in write events.
                let _ = sessions.reregister_writable(&poll, event.token());
            }); 

            // Write:
            // Write if the `Event` is writable..
            // Like `try_read`, `try_write` takes a reference to `poll` and `event`
            // for reregistering intersts and checking if the event is writable.
            //
            // `map` takes a closure that returns the byte slice to be written to the socket.
            // In this case we take the first element in the `write_queue` and return that
            // in the form of a byte slice.
            //
            // `done` happens once and only once if one or more writes were successful.
            //
            // The actual writing of data to the socket happens between `map` and `done`.
            // The write will happen one or more times until either:
            // a) The internal write buffer of the stream (not the `Session`) is full, at which point 
            //    zero bytes will be written. Unlike a read event this does not drop the socket
            //    as it could be a case of no more space in the buffer.
            // b) The connection WouldBlock
            // c) The entire byte slice is written
            // d) The socket is disconnected, at which point the socket will be removed.
            sessions.try_write(&poll, &event).with(|| {
                let obj = &write_queue.get(0).unwrap();
                obj.as_bytes()
            }).done(|bytecount| { // finished writing
                println!("wrote a total of {} bytes", bytecount);
                write_queue.pop_front();
                if write_queue.is_empty() {
                    let _ = sessions.reregister_readable(&poll, event.token());
                } else {
                    let _ = sessions.reregister_writable(&poll, event.token());
                }
            });
        }
    }
}
