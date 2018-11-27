# Sonr

### Note: this is a work in progress and the api will change

Simple Opinionated Networking in Rust

The goal of Sonr is to provide a light weight networking library that makes it
easy to get started writing network applications in Rust, and have a reasonably
low barrier to entry.

It is built on top of Mio.

*  [API docs](https://hagsteel.github.io/sonr/)
*  [Examples](https://github.com/hagsteel/sonr/tree/master/examples)

## Setup

```
[dependencies]
sonr = {git = "https://github.com/hagsteel/sonr" }

```

## Example

```rust
static RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Encoding: UTF-8\nContent-Length: 126\nServer: Sonr example http server\nAccept-Ranges: bytes\nConnection: close\n\n<html> <head> <title>An Example Page</title> </head> <body> Hello World, this is a very simple HTML document.  </body> </html>";

const SERVER_TOKEN: Token = Token(0);

fn main() -> Result<()> {
    let poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let listener = tcp_listener("127.0.0.1", 5000)?;
    let mut server = Server::new(listener, SERVER_TOKEN, &poll);
    server.listen()?;
    let mut sessions = Sessions::with_capacity_and_offset(10_000, 1);

    loop {
        poll.poll(&mut events, None);

        for event in &events {
            if event.token() == SERVER_TOKEN {
                let stream = server.accept()?;
                let connection: TcpConnection = stream.into();
                let session = Session::new(connection);
                let (token, session) = sessions.add(&poll, session)?;
                session.reregister_readable(&poll, token);
                continue
            }

            // Read the request
            sessions.try_read(&poll, &event).and_then(|_buf| {
                // decode the request
            }).done(|_| {
                sessions.reregister_writable(&poll, event.token());
            });

            sessions.try_write(&poll, &event).with(|| {
                &RESPONSE
            }).done(|_| {
                // Drop the connection when it's done
                sessions.remove(event.token());
            });
        }
    }


    Ok(())
}
```
