use std::{collections::HashMap, net::{SocketAddr, TcpStream}};

use common::comm::VehicleState;

mod servo;
mod device;
mod state;

struct Data; // placeholder device data

/// Servo's TCP socket address the FC connects to
const SERVO_ADDRESS: &str = "0.0.0.0:00000"; // placeholder address

/// The TCP socket address where new device connections are accepted
const LISTENER_ADDRESS: &str = "0.0.0.0:00000"; // placeholder address

fn main() -> ! {
  // Maybe we could make servo_stream local to servo.rs?
  let mut servo_stream: TcpStream = servo::establish();
  let mut devices: HashMap<SocketAddr, TcpStream> = HashMap::with_capacity(10);
  let mut state: VehicleState = VehicleState::new();

  loop {
    let connections = device::listen();
    devices.extend(connections);
    let data = device::pull(devices.values_mut());
    state::ingest(&mut state, data);
    servo::push(&mut servo_stream, state.clone());
    servo::pull(&mut servo_stream);

    // TODO: sequence/trigger logic outline
  };
}
