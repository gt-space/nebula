use std::collections::HashMap;
use std::sync::Arc;
use common::comm::{Gpio, Pin, PinMode::Output, PinValue::{Low, High}, ADCKind, SamControlMessage};
use once_cell::sync::Lazy;

pub static GPIO_CONTROLLERS: Lazy<Vec<Arc<Gpio>>> = Lazy::new(|| open_controllers());
// use once_cell::sync::OnceCell;

// pub fn get_gpio_controllers() -> &'static Vec<Gpio> {
//   static INSTANCE: OnceCell<Vec<Gpio>> = OnceCell::new();
//   INSTANCE.get_or_init(|| open_controllers())
// }

// controller = floor(GPIO#/32)
// pin = remainder

// channel = 10 : powered = True
pub fn enable_battery_power() {
  // GPIO 61, Pin 72
  let mut pin = GPIO_CONTROLLERS[1].get_pin(29);
  pin.mode(Output);
  pin.digital_write(High);
}

// channel = 10 : powered = False
pub fn disable_battery_power() {
  // GPIO 61, Pin 72
  let mut pin = GPIO_CONTROLLERS[1].get_pin(29);
  pin.mode(Output);
  pin.digital_write(Low);
}

pub fn disable_rbftag() {
  // GPIO 66, Pin 53
  let mut pin = GPIO_CONTROLLERS[2].get_pin(2);
  pin.mode(Output);
  pin.digital_write(Low);
}

pub fn enable_rbftag() {
  // GPIO 66, Pin 53
  let mut pin = GPIO_CONTROLLERS[2].get_pin(2);
  pin.mode(Output);
  pin.digital_write(High);
}

pub fn reco_disable(channel: u8) {
  match channel {
    1 => {
      // P8 GPIO 46 Pin 62
      let mut pin = GPIO_CONTROLLERS[1].get_pin(14);
      pin.mode(Output);
      pin.digital_write(Low);
    }
    2 => {
      // P8 GPIO 65 Pin 64
      let mut pin = GPIO_CONTROLLERS[2].get_pin(1);
      pin.mode(Output);
      pin.digital_write(Low);
    }
    3 => {
      // P8 GPIO 67 Pin 54
      let mut pin = GPIO_CONTROLLERS[1].get_pin(22);
      pin.mode(Output);
      pin.digital_write(Low);
    },
    4 => {
      // P8 GPIO 68 Pin 56
      let mut pin = GPIO_CONTROLLERS[1].get_pin(24);
      pin.mode(Output);
      pin.digital_write(Low);
    }
    _ => println!("Error"),
  }
}

pub fn init_gpio() {
  // set battery enable low
  // set reco enables low
  disable_battery_power();
  enable_rbftag();
  reco_disable(1);
  reco_disable(2);
  reco_disable(3);
  reco_disable(4);

  for chip_select_pin in get_cs_mappings().values_mut() {
    chip_select_pin.digital_write(High); // active low
  }
}

fn open_controllers() -> Vec<Arc<Gpio>> {
  (0..=3).map(Gpio::open_controller).collect()
}

pub fn get_cs_mappings() -> HashMap<ADCKind, Pin> {
  let mut vbat_chip_select: Pin = GPIO_CONTROLLERS[0].get_pin(14);
  vbat_chip_select.mode(Output);
  let mut reco_chip_select: Pin = GPIO_CONTROLLERS[0].get_pin(15);
  reco_chip_select.mode(Output);

  HashMap::from([(ADCKind::VBatUmbCharge, vbat_chip_select),
  (ADCKind::SamAnd5V, reco_chip_select)])
}

pub fn execute(command: SamControlMessage) {
  match command {
    SamControlMessage::ActuateValve{channel, powered} => {
      match channel {
        20 => {
          if powered {
            enable_battery_power();
          } else {
            disable_battery_power();
          }
        },
        _ => {
          eprintln!("Unrecognized Channel: {channel}");
        }
      };
    },
    _ => {
      eprintln!("Unrecognized Command: {command:#?}");
    }
  };
}

// HOW TO ACTIVATE BMS COMMANDS:
// Mapppings settings:
// Text ID (channel) = battey_power (20)
// SensorType = Valve
// Computer = Flight
// NormallyClosed = False
// Board ID = bms-01
// HOW TO SET BMS PROPERTIES
// Open Valve = True
// Close Valve = False