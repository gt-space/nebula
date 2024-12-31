// Documentation
// Datasheet: https://content.u-blox.com/sites/default/files/ZED-F9P-04B_DataSheet_UBX-21044850.pdf
// Interface Description: https://content.u-blox.com/sites/default/files/documents/u-blox-F9-HPG-1.32_InterfaceDescription_UBX-22008968.pdf
// Ublox examples: https://github.com/ublox-rs/ublox/blob/master/examples/ublox-cli/src/main.rs

// TODO: monver for additional testing
// May have to UBX-CFG-PRT for SPI first
// NavPvt Poll request message (I/O port 4 is SPI)

use spidev::{Spidev, SpidevOptions, SpiModeFlags}; // rppal also provides Raspberry Pi SPI interface if needed
use std::io::{Read, Write};
use std::time::Duration;
use ublox::{
  Position, 
  Velocity, 
  Parser, 
  PacketRef, 
  CfgValSetBuilder, 
  cfg_val::CfgVal, 
  CfgPrtSpiBuilder,
  CfgMsgAllPortsBuilder};
use chrono::{DateTime, Utc};
use rppal::gpio::{Gpio, OutputPin}; // Using Raspberry Pi (remember to enable SPI in raspi-config)


pub struct PVT {
  position: Option<Position>,
  velocity: Option<Velocity>,
  time: Option<DateTime<Utc>>
}

pub struct GPS {
  spidev: Spidev,
  d_sel: OutputPin, // SPI is disabled by default, d_sel needs to be set to 0 to enable SPI
  cs_pin: OutputPin,
  parser: Parser
}

impl GPS {
  pub fn new(bus: &str, d_sel_pin: u8, cs_pin: u8) -> std::io::Result<Self> {
    // Initialize the SPI device
    let mut spidev = Spidev::open(bus)?;
    // See Datasheet Section 5.2
    let options = SpidevOptions::new()
      .bits_per_word(8)
      .max_speed_hz(1_000_000) // 1 MHz to be safe
      .mode(SpiModeFlags::SPI_MODE_0); // Mode 0
    spidev.configure(&options)?;

    // Initialize the ublox parser
    let mut parser = Parser::default();

    // Configure GPIO pins
    let gpio = Gpio::new()?;
    let mut d_sel = gpio.get(d_sel_pin)?.into_output();
    let mut cs_pin = gpio.get(cs_pin)?.into_output();

    // Set initial pin states
    d_sel.set_low(); // Enable SPI by setting d_sel to Low
    cs_pin.set_high(); // Deassert CS (inactive)

    Ok(GPS {spidev, d_sel, cs_pin, parser})
  }

  // Configures the GPS module to use only the GPS constellation
  // This is to get faster refresh rates and based on Aras' RFC
  // NOTE: Not required for now because antenna only picks up GPS signals
  // pub fn configure_gps_constellation(&mut self) -> std::io::Result<()> {
  //   let cfg_gnss = self.build_cfg_gnss_message()?;
  //   self.select_chip()?;
  //   self.spidev.write(&cfg_gnss)?;
  //   self.deselect_chip()?;
  //   self.wait_for_ack()?;
  //   println!("Configured GPS constellation only.");
  //   Ok(())
  // }

  // Builds the UBX-CFG-GNSS message for configuring the GPS constellation
  // See Interface Description Section 1.3, 3.10.26, 6.2, 6.3
  // NOTE: Not required for now because antenna only picks up GPS signals
  // fn build_cfg_gnss_message(&self) -> Vec<u8> {
  //   let packet: [u8; 28] = CfgValSetBuilder {
  //     version: 0,
  //     layers: 0b001, // Updating configuration in RAM layer. This means GPS constellation will have to be configured on every startup.
  //     reserved1: 0,
  //     cfg_data: &[
  //       CfgVal::SignalGpsEna(true),
  //       CfgVal::SignalGpsL1caEna(true),
  //       CfgVal::SignalGpsL2cEna(true), 
  //       CfgVal::SignalGalEna(false),
  //       CfgVal::SignalGalE1Ena(false),
  //       CfgVal::SignalGalE5bEna(false),
  //       CfgVal::SignalBdsEna(false),
  //       CfgVal::SignalBdsB1Ena(false),
  //       CfgVal::SignalBdsB2Ena(false),
  //       CfgVal::SignalQzssEna(false),
  //       CfgVal::SignalQzssL1caEna(false),
  //       CfgVal::SignalQzssL2cEna(false),
  //       CfgVal::SignalGloEna(false),   
  //       CfgVal::SignalGloL1Ena(false),
  //       CfgVal::SignalGLoL2Ena(false),
  //     ]
  //   }.into_byte_array(); // This might cause problems; essentially the struct needs to be converted into a byte array
  //   // See src code for into_byte_array() from other Builder types.
  //   // If using ublox doesn't work, we can also manually create the byte array.
  //   packet
  // }

  // Polls the GPS module for a PVT (Position, Velocity, Time) message
  // See Interface Description Section 3.15.13
  pub fn poll_pvt(&mut self) -> Option<PVT> {
    let mut pvt = PVT {
      position: None,
      velocity: None,
      time: None,
    };

    self.read_packets(|packet| match packet {
      PacketRef::NavPvt(sol) => {
        let has_time = sol.fix_type() == GpsFix::Fix3D
          || sol.fix_type() == GpsFix::GPSPlusDeadReckoning
          || sol.fix_type() == GpsFix::TimeOnlyFix;
        let has_posvel = sol.fix_type() == GpsFix::Fix3D
          || sol.fix_type() == GpsFix::GPSPlusDeadReckoning;

        if has_posvel {
          let pos: Position = (&sol).into();
          let vel: Velocity = (&sol).into();

          println!(
            "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
            pos.lat, pos.lon, pos.alt
          );
          println!(
            "Speed: {:.2} m/s Heading: {:.2} degrees",
            vel.speed, vel.heading
          );
          println!("Sol: {:?}", sol);

          pvt.position = Some(pos);
          pvt.velocity = Some(vel);
        }

        if has_time {
          if let Ok(time) = (&sol).try_into() {
            let time: DateTime<Utc> = time;
            println!("Time: {:?}", time);
            pvt.time = Some(time);
          } else {
            println!("Could not parse NAV-PVT time field to UTC");
          }
        }
      }
      _ => {
        println!("{:?}", packet); // some other packet
      }
    });

    if pvt.position.is_some() || pvt.velocity.is_some() || pvt.time.is_some() {
      Some(pvt)
    } else {
      None
    }
}

  // reads packets from the SPI bus 
  fn read_packets<T: FnMut(PacketRef)>(&mut self, mut cb: T) -> std::io::Result<()> {
    loop {
      const MAX_PAYLOAD_LEN: usize = 1240;
      let mut local_buf = [0; MAX_PAYLOAD_LEN];
      self.select_chip()?;
      let nbytes = self.spidev.read(&mut buffer)?;
      self.deselect_chip()?;
      if nbytes == 0 {
        break;
      }

      // parser.consume adds the buffer to its internal buffer, and
      // returns an iterator-like object we can use to process the packets
      let mut it = self.parser.consume(&local_buf[..nbytes]);
      loop {
        match it.next() {
          Some(Ok(packet)) => {
            cb(packet);
          },
          Some(Err(_)) => {
            // Received a malformed packet, ignore it
          },
          None => {
            // We've eaten all the packets we have
            break;
          },
        }
      }
    }
    Ok(())
  }

  fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
    let mut found_packet = false;
    while !found_packet {
      self.read_packets(|packet| {
        if let PacketRef::AckAck(ack) = packet {
          if ack.class() == T::CLASS && ack.msg_id() == T::ID {
            found_packet = true;
          }
        }
      })?;
    }
    Ok(())
  }

  // Calculates the UBX checksum for a given payload
  // Useful if we need to manually construct packets
  // See Interface Description Section 3.4
  fn calculate_checksum(&self, payload: &[u8]) -> (u8, u8) {
      let mut ck_a = 0u8;
      let mut ck_b = 0u8;
      for byte in payload {
          ck_a = ck_a.wrapping_add(*byte);
          ck_b = ck_b.wrapping_add(ck_a);
      }
      (ck_a, ck_b)
  }

  // Asserts the chip select line (active low).
  fn select_chip(&mut self) -> std::io::Result<()> {
    self.cs_pin.set_low();
    Ok(())
  }

  // Deasserts the chip select line (inactive high).
  fn deselect_chip(&mut self) -> std::io::Result<()> {
    self.cs_pin.set_high();
    Ok(())
  }
}

