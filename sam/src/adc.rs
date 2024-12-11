use jeflog::fail;
use spidev::spidevioctl::SpidevTransfer;
use spidev::Spidev;
use std::sync::Arc;
use std::{thread, time};

use std::collections::HashMap;
use std::rc::Rc;

use crate::gpio::{
  Gpio,
  Pin,
  PinMode::{Input, Output},
  PinValue::{High, Low},
};
use crate::tc::typek_convert;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Measurement {
  CurrentLoopPt,
  VValve,
  IValve,
  VPower,
  IPower,
  Tc1,
  Tc2,
  DiffSensors,
  // Changed Rtd to Rtd1, Rtd2, and Rtd3 in same fashion as TC's for rev4 ground
  Rtd1,
  Rtd2,
  Rtd3
}

pub enum ADCEnum {
  ADC(ADC),
  OnboardADC,
}


pub struct ADC {
  pub measurement: Measurement,
  pub spidev: Rc<Spidev>,
  ambient_temp: f64,
  gpio_mappings: Rc<HashMap<Measurement, Pin>>,
  drdy_mappings: Rc<HashMap<Measurement, Pin>>,
}

impl ADC {
  // Constructs a new instance of an Analog-to-Digital Converter
  pub fn new(
    measurement: Measurement,
    spidev: Rc<Spidev>,
    gpio_mappings: Rc<HashMap<Measurement, Pin>>,
    drdy_mappings: Rc<HashMap<Measurement, Pin>>,
  ) -> ADC {
    ADC {
      measurement,
      spidev,
      ambient_temp: 0.0,
      gpio_mappings,
      drdy_mappings,
    }
  }

  // not called anywhere
  pub fn cs_mappings() -> HashMap<Measurement, usize> {
    let mut cs_gpios: HashMap<Measurement, usize> = HashMap::new();
    cs_gpios.insert(Measurement::CurrentLoopPt, 30);
    cs_gpios.insert(Measurement::IValve, 73); // changed
    cs_gpios.insert(Measurement::VValve, 75); // changed
    // cs_gpios.insert(Measurement::VPower, 13);
    // cs_gpios.insert(Measurement::IPower, 15);
    // cs_gpios.insert(Measurement::Tc1, 10);
    // cs_gpios.insert(Measurement::Tc2, 20);
    cs_gpios.insert(Measurement::DiffSensors, 16);
    // cs_gpios.insert(Measurement::Rtd, 11);

    cs_gpios
  }

  // DO NOT USE THIS FUNCTION
  // pub fn init_gpio(&mut self, prev_adc: Option<Measurement>) {
  //   // pull old adc HIGH
  //   if let Some(old_adc) = prev_adc {
  //     if let Some(pin) = self.gpio_mappings.get(&old_adc) {
  //       pin.digital_write(High);
  //     }
  //   }

  //   // pull new adc LOW
  //   if let Some(pin) = self.gpio_mappings.get(&self.measurement) {
  //     pin.digital_write(Low);
  //   }
  // }

  // selects current ADC
  pub fn pull_cs_high_active_low(&mut self) {
    if let Some(pin) = self.gpio_mappings.get(&self.measurement) {
      pin.digital_write(High);
    }
  }

  // deselects current ADC
  pub fn pull_cs_low_active_low(&mut self) {
    if let Some(pin) = self.gpio_mappings.get(&self.measurement) {
      pin.digital_write(Low);
    }
  }

  pub fn poll_data_ready(&mut self) {
    // poll the data ready pin till low (active low)
    let drdy_pin = self.drdy_mappings.get(&self.measurement).unwrap();

    loop {
      let pin_value = drdy_pin.digital_read();
      if pin_value == Low {
        break;
      }
    }
  }

  pub fn init_regs(&mut self) {
    // Read initial registers
    self.read_regs(0, 17);

    // delay for at least 4000*clock period
    // println!("Delaying for 1 second");
    //thread::sleep(time::Duration::from_millis(100));

    // Write to registers
    match self.measurement {
      Measurement::CurrentLoopPt
      | Measurement::VPower
      | Measurement::IPower
      | Measurement::IValve
      | Measurement::VValve => {
        self.write_reg(0x03, 0x00);
        self.write_reg(0x04, 0x1E);
        // self.write_reg(0x08, 0x40);
        // self.write_reg(0x08, 0x00);
        self.write_reg(0x05, 0x0A);
      }

      Measurement::Rtd1 | Measurement::Rtd2 | Measurement::Rtd3 => {
        self.write_reg(0x03, 0x00);
        self.write_reg(0x04, 0x1E);
        // self.write_reg(0x06, 0x47);
        self.write_reg(0x06, 0x47);
        self.write_reg(0x07, 0x50);
      }

      Measurement::Tc1 | Measurement::Tc2 | Measurement::DiffSensors => {
        self.write_reg(0x03, 0x0D);
        self.write_reg(0x04, 0x1E);
        self.write_reg(0x05, 0x0A);
      }

    }

    // delay for at least 4000*clock period
    // println!("Delaying for 1 second");
    //thread::sleep(time::Duration::from_millis(100));

    // Read registers
    self.read_regs(0, 17);
  }

  pub fn reset_status(&mut self) {
    let tx_buf_reset = [0x06];
    let mut transfer = SpidevTransfer::write(&tx_buf_reset);
    let _status = self.spidev.transfer(&mut transfer);
  }

  pub fn start_conversion(&mut self) {
    let tx_buf_rdata = [0x08];
    let mut rx_buf_rdata = [0x00];
    let mut transfer =
      SpidevTransfer::read_write(&tx_buf_rdata, &mut rx_buf_rdata);
    let _status = self.spidev.transfer(&mut transfer);
    thread::sleep(time::Duration::from_millis(1));
  }

  pub fn self_calibrate(&mut self) {
    let tx_buf_rdata = [0x19];
    let mut rx_buf_rdata = [0x00];
    let mut transfer =
      SpidevTransfer::read_write(&tx_buf_rdata, &mut rx_buf_rdata);
    let _status = self.spidev.transfer(&mut transfer);
    thread::sleep(time::Duration::from_millis(1000));
  }

  pub fn read_regs(&mut self, reg: u8, num_regs: u8) {
    let mut tx_buf_readreg = [0x00; 20];
    let mut rx_buf_readreg = [0x00; 20];
    tx_buf_readreg[0] = 0x20 | reg;
    tx_buf_readreg[1] = num_regs;
    let mut transfer =
      SpidevTransfer::read_write(&tx_buf_readreg, &mut rx_buf_readreg);
    let _status = self.spidev.transfer(&mut transfer);

    println!("{:?} regs: {:?}", self.measurement, rx_buf_readreg);
    if rx_buf_readreg.iter().all(|&byte| byte == 0) {
      fail!("Failed to write and read correct register values");
    }
  }

  pub fn write_reg(&mut self, reg: u8, data: u8) {
    let tx_buf_writereg = [0x40 | reg, 0x00, data];
    let mut rx_buf_writereg = [0x40, 0x00, 0x00];
    let mut transfer =
      SpidevTransfer::read_write(&tx_buf_writereg, &mut rx_buf_writereg);
    let _status = self.spidev.transfer(&mut transfer);
  }

  pub fn get_adc_reading(&mut self, iteration: u64) -> (f64, f64) {
    if self.measurement == Measurement::Tc1 || self.measurement == Measurement::Tc2
    {
      // can't use data ready for these
      // thread::sleep(time::Duration::from_micros(700));
    } else {
      self.poll_data_ready();
    }
    let val = self.test_read_individual(iteration);

    // let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    // let unix_timestamp = start.as_secs_f64();

    let unix_timestamp = 0.0; // change this!

    (val, unix_timestamp)
  }

  pub fn write_iteration(&mut self, iteration: u64) {
    match self.measurement {
      Measurement::CurrentLoopPt => match iteration % 6 {
        0 => {
          self.write_reg(0x02, 0x0C);
        }
        1 => {
          self.write_reg(0x02, 0x10 | 0x0C);
        }
        2 => {
          self.write_reg(0x02, 0x20 | 0x0C);
        }
        3 => {
          self.write_reg(0x02, 0x30 | 0x0C);
        }
        4 => {
          self.write_reg(0x02, 0x40 | 0x0C);
        }
        5 => {
          self.write_reg(0x02, 0x50 | 0x0C);
        }
        _ => fail!("Failed register write — could not mod iteration"),
      },

      Measurement::VValve => match iteration % 6 {
        0 => {
          self.write_reg(0x02, 0x0C);
        }
        1 => {
          self.write_reg(0x02, 0x10 | 0x0C);
        }
        2 => {
          self.write_reg(0x02, 0x20 | 0x0C);
        }
        3 => {
          self.write_reg(0x02, 0x30 | 0x0C);
        }
        4 => {
          self.write_reg(0x02, 0x40 | 0x0C);
        }
        5 => {
          self.write_reg(0x02, 0x50 | 0x0C);
        }
        _ => fail!("Failed register write — could not mod iteration"),
      },

      Measurement::IValve => match iteration % 6 {
        0 | 1 => {
          self.write_reg(0x02, 0x0C);
        },

        2 | 3 => {
          self.write_reg(0x02, 0x10 | 0x0C);
        },

        4 | 5 => {
          self.write_reg(0x02, 0x20 | 0x0C);
        },

        _ => fail!("Failed register write — could not mod iteration"),
      },

      Measurement::Rtd1 | Measurement::Rtd2 | Measurement::Rtd3 => match iteration % 2 {
        0 => {
          self.write_reg(0x02, 0x12);
          self.write_reg(0x05, 0x12);
        }
        1 => {
          self.write_reg(0x02, 0x34);
          self.write_reg(0x05, 0x16);
        }
        _ => fail!("Failed register write — could not mod iteration"),
      },

      Measurement::DiffSensors => match iteration % 2 {
        0 => {
          self.write_reg(0x02, 0x01);
        },

        1 => {
          self.write_reg(0x02, 0x23);
        },

        _ => fail!("Failed register write — could not mod iteration"),
      },

      _ => fail!("Invalid measurement provided")
    }
  }

  pub fn test_read_individual(&mut self, iteration: u64) -> f64 {
    let tx_buf_rdata = [0x12, 0x00, 0x00];
    let mut rx_buf_rdata = [0x00, 0x00, 0x00];
    let mut transfer =
      SpidevTransfer::read_write(&tx_buf_rdata, &mut rx_buf_rdata);
    let _status = self.spidev.transfer(&mut transfer);
    let value: i16 = ((rx_buf_rdata[1] as i16) << 8) | (rx_buf_rdata[2] as i16);

    let mut reading;

    match self.measurement {
      Measurement::CurrentLoopPt => {
        reading = (value as f64) * (2.5 / ((1 << 14) as f64));
        //println!("valve {:?} I: {:?}", (iteration % 6) + 1, reading);
      }

      Measurement::IValve => {
        reading  = (value as f64) * (2.5 / ((1 << 15) as f64));
        reading = (reading * 1200.0) / 1000.0; // different between rev4 ground and rev4 flight. flight is 1560 for the resistor
      }

      Measurement::VPower | Measurement::VValve => {
        reading =
          (value as f64) * (2.5 / ((1 << 15) as f64)) * 11.0; // 0
                                                                               // ref
                                                                               // println!("{:?}: {:?}", (iteration % 5) + 1, reading);
                                                                               //println!("valve {:?} V: {:?}", (iteration % 6) + 1, reading);
      }
      Measurement::IPower => {
        reading = ((value as i32 + 32768) as f64) * (2.5 / ((1 << 15) as f64)); // 2.5 ref
                                                                                // println!("{:?}: {:?}", (iteration % 2) + 1, reading);
      }
      Measurement::Rtd1 | Measurement::Rtd2 | Measurement::Rtd3 => {
        let rtd_resistance = ((value as i32 * 2500) as f64) / ((1 << 15) as f64);

        if rtd_resistance <= 100.0 {
          reading = 0.0014 * rtd_resistance.powi(2) + 2.2521 * rtd_resistance - 239.04;
        } else {
          reading = 0.0014 * rtd_resistance.powi(2) + 2.1814 * rtd_resistance - 230.07;
        }
      }
      Measurement::Tc1 | Measurement::Tc2 => {
        if iteration % 4 == 0 {
          // ambient temp
          reading =
            ((value as i32) as f64) * (2.5 / ((1 << 15) as f64)) * 1000.0;
          let ambient = reading * 0.403 - 26.987;
          self.ambient_temp = ambient;
          self.write_reg(0x09, 0x0); // reset sysmon
          self.write_reg(0x03, 0x0D); // reset PGA gain
        } else {
          // convert
          reading = (value as f64) * (2.5 / ((1 << 15) as f64)) / 0.032; // gain of 32
          reading = (typek_convert(self.ambient_temp as f32, reading as f32)
            + 273.15) as f64;
        }
      }
      Measurement::DiffSensors => {
        reading =
          ((value as f64) * (2.5 / ((1 << 15) as f64)) / 0.032) / 1000.0; // gain of 32
                                                                          // println!("{:?}: {:?}", (iteration % 3) + 1, reading);
      }
    }

    println!("{:?} [{iteration}]: {reading} ({value})", self.measurement);
    reading
  }
}

pub fn open_controllers() -> Vec<Arc<Gpio>> {
  (0..=3).map(Gpio::open).collect()
}

pub fn gpio_controller_mappings( // --> whats on the board
  controllers: &[Arc<Gpio>],
) -> HashMap<Measurement, Pin> {
  // let cl_pin = controllers[0].get_pin(30);
  // cl_pin.mode(Output);

  // let i_valve_pin = controllers[2].get_pin(4);
  // i_valve_pin.mode(Output);

  // let v_valve_pin = controllers[0].get_pin(26);
  // v_valve_pin.mode(Output);

  // let v_power_pin = controllers[2].get_pin(13);
  // v_power_pin.mode(Output);

  // let i_power_pin = controllers[2].get_pin(15);
  // i_power_pin.mode(Output);

  // let tc_1_pin = controllers[0].get_pin(10);
  // tc_1_pin.mode(Output);

  // let tc_2_pin = controllers[0].get_pin(20);
  // tc_2_pin.mode(Output);

  // let diff_pin = controllers[3].get_pin(16);
  // diff_pin.mode(Output);

  // let rtd_pin = controllers[2].get_pin(11);
  // rtd_pin.mode(Output);

    // modified pinout for rev4 ground and added cs diff_pin cuz i fucked up
    let diff_pin = controllers[0].get_pin(30);
    diff_pin.mode(Output);

    let rtd1_pin = controllers[1].get_pin(13);
    rtd1_pin.mode(Output);

    let rtd2_pin = controllers[2].get_pin(5);
    rtd2_pin.mode(Output);

    let rtd3_pin = controllers[2].get_pin(2);
    rtd3_pin.mode(Output);

    let i_valve_pin = controllers[0].get_pin(31);
    i_valve_pin.mode(Output);

    let v_valve_pin = controllers[1].get_pin(16);
    v_valve_pin.mode(Output);

  HashMap::from([
    //(Measurement::CurrentLoopPt, cl_pin), // dedicated CS pin ?
    (Measurement::IValve, i_valve_pin),
    (Measurement::VValve, v_valve_pin),
    //(Measurement::VPower, v_power_pin),
    //(Measurement::IPower, i_power_pin),
    //(Measurement::Tc1, tc_1_pin),
    //(Measurement::Tc2, tc_2_pin),
    (Measurement::DiffSensors, diff_pin),
    (Measurement::Rtd1, rtd1_pin),
    (Measurement::Rtd2, rtd2_pin),
    (Measurement::Rtd3, rtd3_pin),
  ])
}

pub fn data_ready_mappings(
  controllers: &[Arc<Gpio>],
) -> HashMap<Measurement, Pin> {
  // let cl_pin = controllers[1].get_pin(28);
  // cl_pin.mode(Input);

  // let i_valve_pin = controllers[2].get_pin(3);
  // i_valve_pin.mode(Input);

  // let v_valve_pin = controllers[1].get_pin(12);
  // v_valve_pin.mode(Input);

  // let v_power_pin = controllers[2].get_pin(12);
  // v_power_pin.mode(Input);

  // let i_power_pin = controllers[2].get_pin(14);
  // i_power_pin.mode(Input);

  // let diff_pin = controllers[3].get_pin(15);
  // diff_pin.mode(Input);

  // modified pinout for rev4 ground
  let cl_pin = controllers[3].get_pin(17);
  cl_pin.mode(Input);

  let diff_pin = controllers[1].get_pin(28);
  diff_pin.mode(Input);

  let rtd1_pin = controllers[1].get_pin(12);
  rtd1_pin.mode(Input);

  let rtd2_pin = controllers[2].get_pin(4);
  rtd2_pin.mode(Input);

  let rtd3_pin = controllers[2].get_pin(3);
  rtd3_pin.mode(Input);

  let i_valve_pin = controllers[1].get_pin(18);
  i_valve_pin.mode(Input);

  let v_valve_pin = controllers[1].get_pin(19);
  v_valve_pin.mode(Input);

  HashMap::from([
    (Measurement::CurrentLoopPt, cl_pin),
    (Measurement::DiffSensors, diff_pin),
    (Measurement::Rtd1, rtd1_pin),
    (Measurement::Rtd2, rtd2_pin),
    (Measurement::Rtd3, rtd3_pin),
    (Measurement::IValve, i_valve_pin),
    (Measurement::VValve, v_valve_pin),
    // (Measurement::VPower, v_power_pin),
    // (Measurement::IPower, i_power_pin),
  ])
}

pub fn pull_gpios_high(controllers: &[Arc<Gpio>]) { // --> whats on the board
  let pins = vec![
    // controllers[0].get_pin(30),
    // controllers[2].get_pin(4),
    // controllers[0].get_pin(26),
    // controllers[2].get_pin(13),
    // controllers[2].get_pin(15),
    // controllers[0].get_pin(10),
    // controllers[0].get_pin(20),
    // controllers[3].get_pin(16),
    // controllers[2].get_pin(11),
    // controllers[0].get_pin(5),
    // controllers[0].get_pin(13),
    // controllers[0].get_pin(23),
    // controllers[2].get_pin(23),

    // modified pinout for chip selects for rev4 ground and added diff cs cuz i messed up
    controllers[0].get_pin(30),
    controllers[1].get_pin(13),
    controllers[2].get_pin(5),
    controllers[2].get_pin(2),
    controllers[0].get_pin(31),
    controllers[1].get_pin(16),
  ];

  for pin in pins.iter() {
    pin.mode(Output);
    pin.digital_write(High);
  }
}

// check pin numbers
pub fn init_valve_sel_pins(controllers: &[Arc<Gpio>]) -> [Pin; 3] {
  // modified pinout for rev4 ground
  let sel1 = controllers[0].get_pin(22);
  sel1.mode(Output);
  sel1.digital_write(High);

  let sel2 = controllers[0].get_pin(23);
  sel2.mode(Output);
  sel2.digital_write(High);

  let sel3 = controllers[3].get_pin(19);
  sel3.mode(Output);
  sel3.digital_write(High);

  [sel1, sel2, sel3]
}
