use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use mio::Evented;
use mio::tcp::TcpStream;
use mio_uds::UnixStream;
use net2::TcpBuilder;
use net2::unix::UnixTcpBuilderExt;

// use std::sync::Arc;
// use rustls;
// use rustls::ServerSession;

// use servers::tls::make_config_noarc;
use errors::Result;

// Re-exports
pub use mio::tcp::TcpListener;
pub use mio_uds::UnixListener;

// -----------------------------------------------------------------------------
// 		- Listener (which is also Evented) -
// 		S being the stream (TcpStream, UnixStream)
// -----------------------------------------------------------------------------
/// Trait for implementing a listener.
///
/// Useful when creating a new type of listener to use with a [`Server`]
///
/// [`Server`]: struct.Server.html
pub trait Listener: Evented {
    /// An evented stream, e.g a `TcpStream`
    type Stream : Evented;

    /// Accept a stream, commonly used by a `Server` instance.
    fn accept(&self) -> Result<Self::Stream>;
}

// -----------------------------------------------------------------------------
// 		- Tcp Listener -
// -----------------------------------------------------------------------------

impl Listener for TcpListener {
    type Stream = TcpStream;

    fn accept(&self) -> Result<Self::Stream> {
        let (stream, _socket_addr) = self.accept()?;
        Ok(stream)
    }
}

// impl IntoTlsConnection for TcpStream {
//     fn into_tls_connection(self,cert_filepath: &str, private_key_filepath: &str, buffer_size: usize) -> Result<TlsConnection<ServerSession>> {
//         let config = Arc::new(make_config_noarc(cert_filepath, private_key_filepath)?);
//         let tls_session = rustls::ServerSession::new(&config);
//         let connection = TlsConnection::new(self, buffer_size, tls_session)?;
//         Ok(connection)
//     }
// }

/// Create a `TcpListener` from an address and a port.
pub fn tcp_listener(address: &str, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{}", address, port)
        .to_socket_addrs()?
        .next()
        .unwrap_or_else(|| panic!("Failed to resolve: {}", address));

    let builder = match addr {
        SocketAddr::V4(..) => TcpBuilder::new_v4(),
        SocketAddr::V6(..) => TcpBuilder::new_v6(),
    }?;
    builder.reuse_address(true)?;

    #[cfg(unix)]
    builder.reuse_port(true)?;
    builder.bind(&addr)?;
    let listener = TcpListener::from_std(builder.listen(4096)?)?;
    Ok(listener)
}


// -----------------------------------------------------------------------------
// 		- Unix Domain Socket Listener -
// -----------------------------------------------------------------------------
#[cfg(unix)]
impl Listener for UnixListener {
    type Stream = UnixStream;

    fn accept(&self) -> Result<Self::Stream> {
        let (stream, _) = self.accept()?.unwrap();
        Ok(stream)
    }
}

#[cfg(unix)]
/// Create a `UnixListener` for a unix domain socket.
pub fn uds_listener(socket_path: &str) -> Result<UnixListener> {
    Ok(UnixListener::bind(socket_path)?)
}
