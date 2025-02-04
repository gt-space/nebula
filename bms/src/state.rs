use crate::adc::{init_adcs, start_adcs, reset_adcs, poll_adcs};
use crate::{
  command::{init_gpio, GPIO_CONTROLLERS},
  communication::{
    check_and_execute,
    check_heartbeat,
    establish_flight_computer_connection,
    send_data,
  },
};
use ads114s06::ADC;
use common::comm::ADCKind::{SamAnd5V, VBatUmbCharge};
use jeflog::fail;
use std::{
  net::{SocketAddr, UdpSocket},
  time::Instant
};

pub enum State {
  Init,
  Connect(ConnectData),
  MainLoop(MainLoopData),
  Abort(AbortData),
}

pub struct ConnectData {
  adcs: Vec<ADC>,
}

pub struct MainLoopData {
  adcs: Vec<ADC>,
  my_data_socket: UdpSocket,
  my_command_socket: UdpSocket,
  fc_address: SocketAddr,
  then: Instant,
}

pub struct AbortData {
  adcs: Vec<ADC>,
}

impl State {
  pub fn next(self) -> Self {
    match self {
      State::Init => init(),

      State::Connect(data) => connect(data),

      State::MainLoop(data) => main_loop(data),

      State::Abort(data) => abort(data),
    }
  }
}

fn init() -> State {
  init_gpio();

  // VBatUmbCharge
  let adc1: ADC = ADC::new(
    "/dev/spidev0.0",
    GPIO_CONTROLLERS[1].get_pin(28),
    Some(GPIO_CONTROLLERS[0].get_pin(30)),
    VBatUmbCharge,
  )
  .expect("Failed to initialize VBatUmbCharge ADC");

  // SamAnd5V
  let adc2: ADC = ADC::new(
    "/dev/spidev0.0",
    GPIO_CONTROLLERS[1].get_pin(18),
    Some(GPIO_CONTROLLERS[0].get_pin(31)),
    SamAnd5V,
  )
  .expect("Failed to initialize the SamAnd5V ADC");

  let mut adcs: Vec<ADC> = vec![adc1, adc2];
  init_adcs(&mut adcs);

  State::Connect(ConnectData { adcs })
}

fn connect(mut data: ConnectData) -> State {
  let (data_socket, command_socket, fc_address) =
    establish_flight_computer_connection();

  // tell the ADCs to start collecting data
  start_adcs(&mut data.adcs);

  State::MainLoop(MainLoopData {
    adcs: data.adcs,
    my_command_socket: command_socket,
    my_data_socket: data_socket,
    fc_address,
    then: Instant::now(),
  })
}

fn main_loop(mut data: MainLoopData) -> State {
  check_and_execute(&data.my_command_socket);
  let (updated_time, abort_status) =
    check_heartbeat(&data.my_data_socket, data.then);
  data.then = updated_time;

  if abort_status {
    return State::Abort(AbortData { adcs: data.adcs });
  }

  let datapoint = poll_adcs(&mut data.adcs);
  send_data(&data.my_data_socket, &data.fc_address, datapoint);

  State::MainLoop(data)
}

fn abort(mut data: AbortData) -> State {
  fail!("Aborting goodbye!");
  init_gpio();
  /* init_gpio turns off all chip selects but reset_adcs makes use of them
  again. However with the ADC driver that reset_adcs uses, each chip select
  will be turned off after the ADC is done being communicated with. init_gpio
  needs to turn off all chip selects at the start so its mainly code reuse
   */
  reset_adcs(&mut data.adcs); // reset ADC pin muxing and stop collecting data
  State::Connect(ConnectData { adcs: data.adcs })
}
