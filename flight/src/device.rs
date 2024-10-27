use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::LazyLock};
use super::{Data, LISTENER_ADDRESS};

static DEVICE_LISTENER: LazyLock<TcpListener> = LazyLock::new(|| {
  TcpListener::bind(LISTENER_ADDRESS)
    .expect("Could not bind to device socket address.")
});

/// Returns all newfound device streams
pub(super) fn listen() -> Vec<(SocketAddr, TcpStream)> {
  panic!("device::listen: Not Implemented!");
}

/// Collects all incoming data from device streams
pub(super) fn pull<'a, T>(devices: T) -> Vec<Data>
where 
  T: Iterator<Item = &'a mut TcpStream>,
{
  panic!("device::pull: Not Implemented!");
}