use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[cfg(feature = "rusqlite")]
use rusqlite::{
  types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
  ToSql,
};

mod sam;
pub use sam::*;

mod gui;
pub use gui::*;

pub mod gpio;

impl fmt::Display for Unit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Amps => "A",
        Self::Psi => "psi",
        Self::Kelvin => "K",
        Self::Pounds => "lbf",
        Self::Volts => "V",
      }
    )
  }
}

/// Holds a single measurement for either a sensor or valve.
///
/// This enum simply wraps two other types, `SensorMeasurement` and
/// `ValveMeasurement`. The reason to keep this in separate structs instead of
/// properties of the variants is that these values often need to passed around
/// independently in flight code, and enum variant properties are not mutable
/// without reconstructing the variant. This is annoying. Essentially, this
/// looks like bad / less readable code but is necessary, and convenience
/// constructs are provided to make code cleaner.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Measurement {
  /// The raw value associated with the measurement.
  pub value: f64,

  /// The unit associated with the measurement.
  pub unit: Unit,
}

impl fmt::Display for Measurement {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:.3} {}", self.value, self.unit)
  }
}

/// Holds the state of the vehicle using `HashMap`s which convert a node's name
/// to its state.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VehicleState {
  /// Holds the actual and commanded states of all valves on the vehicle.
  pub valve_states: HashMap<String, CompositeValveState>,

  /// Holds the latest readings of all sensors on the vehicle.
  pub sensor_readings: HashMap<String, Measurement>,
}

impl VehicleState {
  /// Constructs a new, empty `VehicleState`.
  pub fn new() -> Self {
    VehicleState::default()
  }
}

/// Used in a `NodeMapping` to determine which computer the action should be
/// sent to.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, MaxSize, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Computer {
  /// The flight computer
  Flight,

  /// The ground computer
  Ground,
}

#[cfg(feature = "rusqlite")]
impl ToSql for Computer {
  fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
    // see the ChannelType ToSql comment for details
    let mut json = serde_json::to_string(&self)
      .expect("failed to serialize ChannelType into JSON");

    json.pop();
    json.remove(0);

    Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(json)))
  }
}

#[cfg(feature = "rusqlite")]
impl FromSql for Computer {
  fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
    if let ValueRef::Text(text) = value {
      // see the ChannelType ToSql comment for details
      let mut json = vec![b'"'];
      json.extend_from_slice(text);
      json.push(b'"');

      let channel_type = serde_json::from_slice(&json)
        .map_err(|error| FromSqlError::Other(Box::new(error)))?;

      Ok(channel_type)
    } else {
      Err(FromSqlError::InvalidType)
    }
  }
}

/// The mapping of an individual node.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NodeMapping {
  /// The text identifier, or name, of the node.
  pub text_id: String,

  /// A string identifying an individual board, corresponding to the hostname
  /// sans ".local".
  pub board_id: String,

  /// The channel type of the node, such as "valve".
  pub sensor_type: SensorType,

  /// A number identifying which channel on the SAM board controls the node.
  pub channel: u32,

  /// Which computer controls the SAM board, "flight" or "ground".
  pub computer: Computer,

  // the optional parameters below are only needed for sensors with certain
  // channel types if you're wondering why these are not kept with the
  // ChannelType variants, that is because those variants are passed back from
  // the SAM boards with data measurements. the SAM boards have no access to
  // these factors and even if they did, it would make more sense for them to
  // just convert the measurements directly.
  //
  // tl;dr this is correct and reasonable.
  /// The maximum value reading of the sensor.
  /// This is only used for sensors with channel type CurrentLoop or
  /// DifferentialSignal.
  pub max: Option<f64>,

  /// The minimum value reading of the sensor.
  /// This is only used for sensors with channel type CurrentLoop or
  /// DifferentialSignal.
  pub min: Option<f64>,

  /// The calibrated offset of the sensor.
  /// This is only used for sensors with channel type PT.
  #[serde(default)]
  pub calibrated_offset: f64,

  /// The threshold, in Amps, at which the valve is considered powered.
  pub powered_threshold: Option<f64>,

  /// Indicator of whether the valve is normally open or normally closed.
  pub normally_closed: Option<bool>,
}

/// A sequence written in Python, used by the flight computer to execute
/// arbitrary operator code.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Sequence {
  /// The unique, human-readable name which identifies the sequence.
  ///
  /// If the name is "abort" specifically, the sequence should be stored by the
  /// recipient and persisted across a machine power-down instead of run
  /// immediately.
  pub name: String,

  /// The script run immediately (except abort) upon being received.
  pub script: String,
}

/// A trigger with a
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Trigger {
  /// The unique, human-readable name which identifies the trigger.
  pub name: String,

  /// The condition upon which the trigger script is run, written in Python.
  pub condition: String,

  /// The script run when the condition is met, written in Python.
  pub script: String,

  /// Whether or not the trigger is active
  pub active: bool,
}

/// A message sent from the control server to the flight computer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum FlightControlMessage {
  /// A set of mappings to be applied immediately.
  Mappings(Vec<NodeMapping>),

  /// A message containing a sequence to be run immediately.
  Sequence(Sequence),

  /// A trigger to be checked by the flight computer.
  Trigger(Trigger),

  /// Instructs the flight computer to stop a sequence named with the `String`
  /// parameter.
  StopSequence(String),

  /// Instructs the flight computer to run an immediate abort.
  Abort,
}
