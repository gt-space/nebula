use std::{net::{TcpListener, TcpStream, SocketAddr}, sync::LazyLock};
use super::{Data, LISTENER_ADDRESS};
use std::io::ErrorKind;
use std::io::Read;



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
  const DATA_BUFFER_SIZE: usize = 1024; // ? 
  let mut collected_data = Vec::new();
  for stream in devices{
    let mut buffer = [0; DATA_BUFFER_SIZE];
    stream.set_nonblocking(true).expect("set_nonblocking call failed");

    match stream.read(&mut buffer) {
      Ok(0) => {
          // handle connection dropped 
          eprintln!("connection has been dropped");
          continue;
      }
      Ok(n) => {
        //how should I convert the data to Data Struct?
        collected_data.push(buffer[..n].to_vec());

      }
      Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
          // No data; should check heartbeat timer here?
          continue;
      }
      Err(e) => {
          eprintln!("Error reading from stream: {}", e);
      }
  }
}

collected_data
}


