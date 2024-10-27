use std:: net::TcpStream;
use common::comm::VehicleState;

/// Establishes a TCP stream with Servo
pub(super) fn establish() -> TcpStream {
  panic!("servo::establish: Not Implemented!");
}

/// Gets new config/mapping information from Servo
pub(super) fn pull(stream: &mut TcpStream) {
  panic!("servo::pull: Not Implemented!");
}

/// Sends new VehicleState to servo
pub(super) fn push(stream: &mut TcpStream, state: VehicleState) {
  panic!("servo::push: Not Implemented!");
}