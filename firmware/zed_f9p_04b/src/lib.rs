//! Documentation
//! Datasheet: https://content.u-blox.com/sites/default/files/ZED-F9P-04B_DataSheet_UBX-21044850.pdf
//! Interface Description: https://content.u-blox.com/sites/default/files/documents/u-blox-F9-HPG-1.32_InterfaceDescription_UBX-22008968.pdf
//! Ublox examples: https://github.com/ublox-rs/ublox/blob/master/examples/ublox-cli/src/main.rs

// TODO: May have to UBX-CFG-PRT for SPI first

use chrono::{DateTime, Utc};
use rppal::gpio::{Gpio, OutputPin}; // Using Raspberry Pi (remember to enable SPI in raspi-config)
use spidev::{SpiModeFlags, Spidev, SpidevOptions}; // rppal also provides Raspberry Pi SPI interface if needed
use std::{
  fmt, io,
  io::{Read, Write},
  thread,
  time::Duration,
};
use ublox::{
  CfgMsgAllPorts, CfgMsgAllPortsBuilder, GpsFix, MonVer, NavPvt, PacketRef,
  Parser, Position, UbxPacketMeta, UbxPacketRequest, Velocity,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct PVT {
  pub position: Option<Position>,
  pub velocity: Option<Velocity>,
  pub time: Option<DateTime<Utc>>,
}

pub struct GPS {
  spidev: Spidev,
  // d_sel pin commented as it is already hardwired to GND on blackbox
  //d_sel: OutputPin, // SPI is disabled by default, d_sel needs to be set to 0 to enable SPI
  cs_pin: OutputPin,
  parser: Parser<Vec<u8>>,
}

// Error type for instantiating GPS
#[derive(Debug)]
pub enum GPSError {
  SPI(io::Error),
  GPIO(rppal::gpio::Error),
  GPSMessage(io::Error),
  Configuration(String),
}

impl fmt::Display for GPSError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      GPSError::SPI(err) => write!(f, "SPI error: {}", err),
      GPSError::GPIO(err) => write!(f, "GPIO error: {}", err),
      GPSError::GPSMessage(err) => write!(f, "GPS Message error {}", err),
      GPSError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
    }
  }
}

impl std::error::Error for GPSError {}

impl From<io::Error> for GPSError {
  fn from(err: io::Error) -> Self {
    GPSError::SPI(err)
  }
}

impl From<rppal::gpio::Error> for GPSError {
  fn from(err: rppal::gpio::Error) -> Self {
    GPSError::GPIO(err)
  }
}

impl GPS {
  pub fn new(
    bus: &str,
    /*d_sel_pin: u8,*/ cs_pin: u8,
  ) -> Result<Self, GPSError> {
    // Initialize the SPI device
    let mut spidev = Spidev::open(bus).map_err(GPSError::SPI)?;
    // See Datasheet Section 5.2
    let mut options = SpidevOptions::new();
    options
      .bits_per_word(8)
      .max_speed_hz(1_000_000) // 1 MHz to be safe
      .mode(SpiModeFlags::SPI_MODE_0); // Mode 0
    spidev.configure(&options).map_err(GPSError::SPI)?;

    // Initialize the ublox parser
    let parser = Parser::default();

    // Configure GPIO pins
    let gpio = Gpio::new()?;
    // let mut d_sel = gpio.get(d_sel_pin)?.into_output();
    let mut cs_pin = gpio.get(cs_pin)?.into_output();

    // Set initial pin states
    // d_sel.set_low(); // Enable SPI by setting d_sel to Low
    cs_pin.set_high(); // Deassert CS (inactive)

    Ok(GPS {
      spidev,
      /*d_sel,*/ cs_pin,
      parser,
    })
  }

  // Configures the GPS module to use only the GPS constellation
  // This is to get faster refresh rates and based on Aras' RFC
  // NOTE: Not required for now because antenna only picks up GPS signals
  // pub fn configure_gps_constellation(&mut self) -> std::io::Result<()> {
  //   let cfg_gnss = self.build_cfg_gnss_message()?;
  //   self.select_chip()?;
  //   self.spidev.write_all(&cfg_gnss)?;
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

  // Send MonVer message (asking for version information)
  // Useful to test SPI communication with the module since it is
  // configuration-independent
  // See Interface Description Section 3.14.15
  pub fn mon_ver(&mut self) -> Result<(), GPSError> {
    self.select_chip()?;
    self.spidev.write_all(
      &UbxPacketRequest::request_for::<MonVer>().into_packet_bytes(),
    )?;
    self.deselect_chip()?;
    let mut found_mon_ver = false;
    thread::sleep(Duration::from_millis(500));
    self
      .read_packets(|packet| match packet {
        PacketRef::MonVer(packet) => {
          found_mon_ver = true;
          println!(
            "SW version: {} HW version: {}; Extensions: {:?}",
            packet.software_version(),
            packet.hardware_version(),
            packet.extension().collect::<Vec<&str>>()
          );
          println!("{:?}", packet);
        }
        _ => {
          println!("{:?}", packet); // some other packet
        }
      })
      .map_err(GPSError::GPSMessage)?;
    if found_mon_ver {
      Ok(())
    } else {
      Err(GPSError::GPSMessage(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No UBX-MON-VER response received from the device.",
      )))
    }
  }

  // NavPvt Poll request message (I/O port id 4 is SPI)
  // Call this first to start receiving NavPvt messages
  // See Interface Description Section 3.10.10
  pub fn nav_pvt_poll_req(&mut self) -> Result<(), GPSError> {
    self.select_chip()?;
    self.spidev.write_all(
      &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 0, 0, 0, 1, 0])
        .into_packet_bytes(),
    )?;
    self.deselect_chip()?;
    self.wait_for_ack::<CfgMsgAllPorts>()?;
    Ok(())
  }

  // Polls the GPS module for a PVT (Position, Velocity, Time) message
  // See Interface Description Section 3.15.13
  pub fn poll_pvt(&mut self) -> Result<Option<PVT>, GPSError> {
    let mut pvt = PVT {
      position: None,
      velocity: None,
      time: None,
    };

    self
      .read_packets(|packet| match packet {
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
      })
      .map_err(GPSError::GPSMessage)?;

    if pvt.position.is_some() || pvt.velocity.is_some() || pvt.time.is_some() {
      Ok(Some(pvt))
    } else {
      Err(GPSError::GPSMessage(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No valid NAV-PVT response received from the device.",
      )))
    }
  }

  // reads packets from the SPI bus
  fn read_packets<T: FnMut(PacketRef)>(
    &mut self,
    mut cb: T,
  ) -> std::io::Result<()> {
    loop {
      const MAX_PAYLOAD_LEN: usize = 1240;
      let mut local_buf = [0; MAX_PAYLOAD_LEN];
      self.select_chip()?;
      let nbytes = self.spidev.read(&mut local_buf)?;
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
          }
          Some(Err(_)) => {
            // Received a malformed packet, ignore it
          }
          None => {
            // We've eaten all the packets we have
            break;
          }
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
  // fn calculate_checksum(&self, payload: &[u8]) -> (u8, u8) {
  //   let mut ck_a = 0u8;
  //   let mut ck_b = 0u8;
  //   for byte in payload {
  //     ck_a = ck_a.wrapping_add(*byte);
  //     ck_b = ck_b.wrapping_add(ck_a);
  //   }
  //   (ck_a, ck_b)
  // }

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
