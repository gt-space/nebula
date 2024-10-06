use clap::ArgMatches;
use common::comm::{
  ChannelType,
  CompositeValveState,
  DataMessage,
  DataPoint,
  Measurement,
  Unit,
  ValveState,
  VehicleState,
};
use common::comm::{Computer, FlightControlMessage};
use jeflog::fail;
use postcard::experimental::max_size::MaxSize;
use std::{
  borrow::Cow,
  io::{self, Read, Write},
  net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
  thread,
  time::Duration,
};

pub fn emulate_flight() -> anyhow::Result<()> {
  let mut flight: TcpStream =
    TcpStream::connect("localhost:5025").expect("Couldn't connect to servo...");

  // send identity message
  let mut identity = [0; Computer::POSTCARD_MAX_SIZE];

  let comp_type: Computer = Computer::Flight;

  if let Err(error) = postcard::to_slice(&comp_type, &mut identity) {
    fail!("Failed to serialize Computer: {error}");
    return Err(error.into());
  }
  flight.write_all(&identity)?;

  let data_socket = UdpSocket::bind("0.0.0.0:0")?;
  data_socket.connect("localhost:7201")?;

  let mut mock_vehicle_state = VehicleState::new();
  mock_vehicle_state.valve_states.insert(
    "BBV".to_owned(),
    CompositeValveState {
      commanded: ValveState::Closed,
      actual: ValveState::Closed,
    },
  );
  mock_vehicle_state.valve_states.insert(
    "SWV".to_owned(),
    CompositeValveState {
      commanded: ValveState::Open,
      actual: ValveState::Open,
    },
  );
  mock_vehicle_state.valve_states.insert(
    "BYE".to_owned(),
    CompositeValveState {
      commanded: ValveState::Closed,
      actual: ValveState::Disconnected,
    },
  );
  mock_vehicle_state.valve_states.insert(
    "HUH".to_owned(),
    CompositeValveState {
      commanded: ValveState::Open,
      actual: ValveState::Undetermined,
    },
  );
  mock_vehicle_state.valve_states.insert(
    "BAD".to_owned(),
    CompositeValveState {
      commanded: ValveState::Closed,
      actual: ValveState::Fault,
    },
  );

  let mut raw = postcard::to_allocvec(&mock_vehicle_state)?;
  postcard::from_bytes::<VehicleState>(&raw).unwrap();
  let mut nothing_count = 0;

  flight
    .set_nonblocking(true)
    .expect("Setting non-blocking failed.");
  let mut buf: [u8; 100_000] = [0; 100_000];
  loop {
    match flight.read(&mut buf) {
      Ok(x) => {
        println!();
        if x == 0 {
          println!("Got 0 bytes. Closing.");
          return Ok(());
        }
        print!("0x");
        for byte in buf.iter().take(x) {
          print!("{:02x}", byte);
        }
        println!();
        println!("Got {} bytes", x);
        match postcard::from_bytes::<FlightControlMessage>(&buf) {
          Err(err) => {
            println!("Error / Unrecognized message from servo : \n {}", err)
          }
          Ok(message) => match message {
            FlightControlMessage::Sequence(sequence) => {
              println!("Received sequence \"{}\" : ", sequence.name);
              for line in sequence.script.split("\n") {
                println!("  {}", line);
              }
            }
            FlightControlMessage::Mappings(mappings) => {
              println!("Received mappings : ");
              for mapping in mappings {
                println!("  {} : Unimplemented display func", mapping.text_id);
              }
            }
            FlightControlMessage::Trigger(trigger) => {
              println!("Received trigger \"{}\" : ", trigger.name);
              println!(
                "  if ({}) [{}]  :\n",
                trigger.condition,
                if trigger.active {
                  "currently active"
                } else {
                  "not currently active"
                }
              );
              for line in trigger.script.split("\n") {
                println!("    {}", line);
              }
            }
            FlightControlMessage::StopSequence(name) => {
              println!("received request to stop sequence {}", name)
            }
            FlightControlMessage::Abort => println!("Received abort request"),
          },
        }
      }
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        if nothing_count == 0 {
          println!("Nothing");
        }
        if nothing_count % 10 == 0 {
          print!(".");
        }
        nothing_count += 1;
      }
      Err(err) => panic!("Error on read {}", err),
    };
    io::stdout().flush()?;

    mock_vehicle_state.sensor_readings.insert(
      "KBPT".to_owned(),
      Measurement {
        value: rand::random::<f64>() * 120.0,
        unit: Unit::Psi,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "WTPT".to_owned(),
      Measurement {
        value: rand::random::<f64>() * 1000.0,
        unit: Unit::Psi,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "BBV_V".to_owned(),
      Measurement {
        value: 2.2,
        unit: Unit::Volts,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "BBV_I".to_owned(),
      Measurement {
        value: 0.01,
        unit: Unit::Amps,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "SWV_V".to_owned(),
      Measurement {
        value: 24.0,
        unit: Unit::Volts,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "SWV_I".to_owned(),
      Measurement {
        value: 0.10,
        unit: Unit::Amps,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "BAD_V".to_owned(),
      Measurement {
        value: 1000.0,
        unit: Unit::Volts,
      },
    );
    mock_vehicle_state.sensor_readings.insert(
      "BAD_I".to_owned(),
      Measurement {
        value: 0.0,
        unit: Unit::Amps,
      },
    );
    raw = postcard::to_allocvec(&mock_vehicle_state)?;

    data_socket.send(&raw)?;
    thread::sleep(Duration::from_millis(10));
  }
}

pub fn emulate_sam(flight: SocketAddr) -> anyhow::Result<()> {
  let socket = UdpSocket::bind("0.0.0.0:0")?;
  socket.connect(flight)?;

  let mut buffer = [0; 1024];
  let data_points = vec![
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::CurrentLoop,
    },
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::RailVoltage,
    },
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::RailCurrent,
    },
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::Rtd,
    },
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::DifferentialSignal,
    },
    DataPoint {
      value: 0.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::Tc,
    },
    DataPoint {
      value: 23.0,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::ValveVoltage,
    },
    DataPoint {
      value: 0.00,
      timestamp: 0.0,
      channel: 1,
      channel_type: ChannelType::ValveCurrent,
    },
  ];

  let board_id = "sam-01";

  let identity = DataMessage::Identity(board_id.to_owned());
  let handshake = postcard::to_slice(&identity, &mut buffer)?;
  socket.send(handshake)?;

  loop {
    let message =
      DataMessage::Sam(board_id.to_owned(), Cow::Borrowed(&data_points));

    let serialized = postcard::to_slice(&message, &mut buffer)?;
    socket.send(serialized)?;

    thread::sleep(Duration::from_millis(1));
  }
}

/// Tool function which emulates different components of the software stack.
pub fn emulate(args: &ArgMatches) -> anyhow::Result<()> {
  let component = args.get_one::<String>("component").unwrap();

  match component.as_str() {
    "flight" => emulate_flight(),
    "sam" => emulate_sam(
      "localhost:4573"
        .to_socket_addrs()?
        .find(|addr| addr.is_ipv4())
        .unwrap(),
    ),
    other => {
      fail!("Unrecognized emulator component '{other}'.");
      Ok(())
    }
  }
}
