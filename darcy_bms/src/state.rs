use std::{borrow::Cow, net::{SocketAddr, ToSocketAddrs, UdpSocket}, process::exit, thread, time::{Duration, Instant}};
use common::comm::{ChannelType, DataPoint, DataMessage, ADCKind, ADCKind::{VBatUmbCharge, SamAnd5V}, Gpio, PinValue::{Low, High}, PinMode::{Input, Output}};
use ads114s06::ADC;
use std::thread::sleep;

use crate::{command::GPIO_CONTROLLERS, communication::{check_and_execute, check_heartbeat, establish_flight_computer_connection, send_data}};
use crate::adc::{init_adcs, poll_adcs};
use crate::command::init_gpio;
use jeflog::{warn, fail, pass};

pub enum State {
  Init,
  Connect(ConnectData),
  MainLoop(MainLoopData),
  Abort(AbortData)
}

pub struct ConnectData {
  adcs: Vec<ADC>,
}

pub struct MainLoopData {
  adcs: Vec<ADC>,
  my_data_socket: UdpSocket,
  my_command_socket: UdpSocket,
  fc_address: SocketAddr,
  then: Instant
}

pub struct AbortData {
  adcs: Vec<ADC>
}

impl State {

  pub fn next(mut self) -> Self {
    match self {
      State::Init => {
        init()
      },

      State::Connect(data) => {
        connect(data)
      },

      State::MainLoop(data) => {
        main_loop(data)
      }

      State::Abort(data) => {
        abort(data)
      }
    }
  }

}

fn init() -> State {
  init_gpio();

  // VBatUmbCharge
  let mut battery_adc: ADC = ADC::new(
    "/dev/spidev0.0",
    GPIO_CONTROLLERS[1].get_pin(17),
    Some(GPIO_CONTROLLERS[0].get_pin(14)),
    VBatUmbCharge
  ).expect("Failed to initialize VBatUmbCharge ADC");

  println!("Battery ADC regs (before init)");
  for (reg, reg_value) in battery_adc.spi_read_all_regs().unwrap().into_iter().enumerate() {
    println!("Reg {:x}: {:08b}", reg, reg_value);
  }
  println!("\n");

  let mut adcs: Vec<ADC> = vec![battery_adc];
  init_adcs(&mut adcs);

  State::Connect(
    ConnectData {
      adcs
    }
  )
}

fn connect(data: ConnectData) -> State {
  let (data_socket, command_socket, fc_address) = establish_flight_computer_connection();

  State::MainLoop(
    MainLoopData {
      adcs: data.adcs,
      my_command_socket: command_socket,
      my_data_socket: data_socket,
      fc_address,
      then: Instant::now()
    }
  )
}

fn main_loop(mut data: MainLoopData) -> State {
  check_and_execute(&data.my_command_socket);
  let (updated_time, abort_status) = check_heartbeat(&data.my_data_socket, data.then);
  data.then = updated_time;

  if abort_status {
    return State::Abort(
      AbortData {
        adcs: data.adcs
      }
    )
  }

  let datapoints: Vec<DataPoint> = poll_adcs(&mut data.adcs);
  send_data(&data.my_data_socket, &data.fc_address, datapoints);

  State::MainLoop(data)
}

fn abort(data: AbortData) -> State {
  fail!("Aborting...");
  init_gpio();
  State::Connect(
    ConnectData {
      adcs: data.adcs
    }
  )
}