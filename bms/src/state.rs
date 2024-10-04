use std::collections::HashMap;
use crate::{command, adc::ADCType};

use crate::gpio::{
  Gpio,
  Pin,
  PinMode::{Input, Output},
  PinValue::{High, Low},
};

#[derive(PartialEq, Debug)]
pub enum State {
  InitGpio,
  EstablishFlightComputerConnection,
  InitAdcs,
  PollAdcs
}


impl State {
  pub fn next(self, data: &mut Data) -> State {
    match self {
      State::InitGpio => {

      },

      State::EstablishFlightComputerConnection => {

      },

      State::InitAdcs => {

      },

      State::PollAdcs => {

      }
    }
  }
}

pub fn init_gpio(gpio_controllers: &[Gpio]) {
  // set battery enable low
  // set sam enable low (disable)
  // set charge enable low (disable)
  // set estop reset low
  command::disable_battery_power(gpio_controllers);
  command::disable_sam_power(gpio_controllers);
  command::disable_charger(gpio_controllers);
  command::set_estop_low(gpio_controllers);
}

pub fn get_chip_select_mappings(gpio_controllers: &[Gpio]) -> HashMap<ADCType, Pin> {
  let mut vbat_umb_charge_chip_select: Pin = gpio_controllers[0].get_pin(30);
  vbat_umb_charge_chip_select.mode(Output);
  let mut sam_and_5v_chip_select: Pin = gpio_controllers[0].get_pin(31);
  sam_and_5v_chip_select.mode(Output);

  HashMap::from([(ADCType::VBatUmbCharge, vbat_umb_charge_chip_select),
  (ADCType::SamAnd5V, sam_and_5v_chip_select)])
}

pub fn get_drdy_mappings(gpio_controllers: &[Gpio]) -> HashMap<ADCType, Pin> {
  let mut vbat_umb_charge_drdy: Pin = gpio_controllers[1].get_pin(28);
  vbat_umb_charge_drdy.mode(Input);
  let mut sam_and_5v_drdy: Pin = gpio_controllers[1].get_pin(18);
  sam_and_5v_drdy.mode(Input);

  HashMap::from([(ADCType::VBatUmbCharge, vbat_umb_charge_drdy), 
  (ADCType::SamAnd5V, sam_and_5v_drdy)])
}