use std::borrow::Cow;

use crate::adc;
use common::comm::DataMessage;
use common::comm::DataPoint;

pub fn serialize_data(
  board_id: String,
  data_points: &Vec<DataPoint>,
) -> Result<Vec<u8>, postcard::Error> {
  let data_message = DataMessage::Sam(board_id, Cow::Borrowed(data_points));
  postcard::to_allocvec(&data_message)
}

pub fn generate_data_point(
  data: f64,
  timestamp: f64,
  iteration: u64,
  measurement: adc::Measurement,
) -> DataPoint {
  DataPoint {
    value: data,
    timestamp,
    channel: iteration_to_node_id(measurement, iteration).unwrap(),
    channel_type: measurement_to_channel_type(
      iteration_to_node_id(measurement, iteration).unwrap(),
      measurement,
    )
    .unwrap(),
  }
}

// modified for rev4 ground
fn iteration_to_node_id(
  measurement: adc::Measurement,
  iteration: u64,
) -> Option<u32> {
  let node_id = match measurement {
    adc::Measurement::CurrentLoopPt
    | adc::Measurement::IValve
    | adc::Measurement::VValve => iteration % 6 + 1,
    adc::Measurement::VPower => iteration % 5 + 1,
    adc::Measurement::IPower => iteration % 2 + 1,
    adc::Measurement::DiffSensors => iteration % 2 + 1,
    adc::Measurement::Tc1 => iteration % 4,
    adc::Measurement::Tc2 => iteration % 4 + 3,
    adc::Measurement::Rtd1 => iteration % 2 + 1,
    adc::Measurement::Rtd2 => iteration % 2 + 3,
    adc::Measurement::Rtd3 => iteration % 2 + 5,
  };

  u32::try_from(node_id).ok()
}

// modified for rev4 ground
fn measurement_to_channel_type(
  node_id: u32,
  measurement: adc::Measurement,
) -> Option<common::comm::ChannelType> {
  match (node_id, measurement) {
    (_, adc::Measurement::CurrentLoopPt) => {
      Some(common::comm::ChannelType::CurrentLoop)
    }
    (_, adc::Measurement::VValve) => {
      Some(common::comm::ChannelType::ValveVoltage)
    }
    (_, adc::Measurement::IValve) => {
      Some(common::comm::ChannelType::ValveCurrent)
    }
    // (0, adc::Measurement::VPower) =>
    // Some(common::comm::ChannelType::RailVoltage),
    // (1, adc::Measurement::VPower) =>
    // Some(common::comm::ChannelType::RailVoltage),
    // (2, adc::Measurement::VPower) =>
    // Some(common::comm::ChannelType::RailVoltage), // Digital
    // (3, adc::Measurement::VPower) =>
    // Some(common::comm::ChannelType::RailVoltage), // Analog
    // (4, adc::Measurement::VPower) =>
    // Some(common::comm::ChannelType::RailVoltage),
    // (0, adc::Measurement::IPower) =>
    // Some(common::comm::ChannelType::RailCurrent), // 24V
    // (1, adc::Measurement::IPower) =>
    // Some(common::comm::ChannelType::RailCurrent), // 5V
    (_, adc::Measurement::VPower) => {
      Some(common::comm::ChannelType::RailVoltage)
    }
    (_, adc::Measurement::IPower) => {
      Some(common::comm::ChannelType::RailCurrent)
    } // 24V
    (_, adc::Measurement::DiffSensors) => {
      Some(common::comm::ChannelType::DifferentialSignal)
    }
    (_, adc::Measurement::Rtd1) => Some(common::comm::ChannelType::Rtd),
    (_, adc::Measurement::Rtd2) => Some(common::comm::ChannelType::Rtd),
    (_, adc::Measurement::Rtd3) => Some(common::comm::ChannelType::Rtd),
    (_, adc::Measurement::Tc1) => Some(common::comm::ChannelType::Tc),
    (_, adc::Measurement::Tc2) => Some(common::comm::ChannelType::Tc),
  }
}
