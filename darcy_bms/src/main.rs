// Copied Vespula BMS code from luna/bms directory

pub mod command;
pub mod adc;
pub mod state;
pub mod communication;

use std::{borrow::Cow, net::{SocketAddr, ToSocketAddrs, UdpSocket}, process::exit, thread, time::{Duration, Instant}};
use command::execute;
use common::comm::{ChannelType, DataMessage, DataPoint, Gpio, PinValue::{Low, High}, SamControlMessage, ADCKind, ADCKind::{VBatUmbCharge, SamAnd5V}};
use jeflog::{warn, fail, pass};
use ads114s06::ADC;
use adc::{init_adcs, poll_adcs};

const FC_ADDR: &str = "server-01";
const BMS_ID: &str = "bms-01";
const COMMAND_PORT: u16 = 8378;
const HEARTBEAT_TIME_LIMIT: Duration = Duration::from_millis(250);

fn main() {
  let mut state = state::State::Init;
  
  loop {
    state = state.next();
  }
}