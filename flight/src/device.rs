use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::LazyLock};
use super::{Data, LISTENER_ADDRESS};
use std::io::ErrorKind;


static DEVICE_LISTENER: LazyLock<TcpListener> = LazyLock::new(|| {
 TcpListener::bind(LISTENER_ADDRESS)
   .expect("Could not bind to device socket address.")
});


/// Returns all newfound device streams
pub(super) fn listen() -> Vec<(SocketAddr, TcpStream)> {


    let mut connections = Vec::new();
     // Set the listener to non-blocking mode to avoid blocking the main thread
    DEVICE_LISTENER.set_nonblocking(true).expect("Failed to set non-blocking mode.");
     loop {
        match DEVICE_LISTENER.accept() {
            Ok((stream, addr)) => {
                // If a connection is accepted, add it to the list
                connections.push((addr, stream));
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // Not ready yet so we will check in next stream
                break;
            }
            Err(e) => panic!("encountered IO error: {e}"),         
        }
    }
    connections
 }


/// Collects all incoming data from device streams
pub(super) fn pull<'a, T>(devices: T) -> Vec<Data>
where
 T: Iterator<Item = &'a mut TcpStream>,
{
 panic!("device::pull: Not Implemented!");
}
