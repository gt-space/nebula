use super::*;
use crate::server::error::ServerError;
use common::comm::CompositeValveState;
use std::{
  collections::{HashMap, VecDeque},
  time::Duration,
  vec::Vec,
};

use common::comm::{Measurement, Sequence};
use std::string::String;
use tui_input::Input;

#[derive(Clone, Debug)]
pub struct NamedValue<T: Clone> {
  pub name: String,
  pub value: T,
}

impl<T: Clone> NamedValue<T> {
  pub fn new(new_name: String, new_value: T) -> NamedValue<T> {
    NamedValue {
      name: new_name,
      value: new_value,
    }
  }
}

impl<T: Clone + PartialEq> PartialEq for NamedValue<T> {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.value == other.value
  }
}

#[derive(Clone, Debug)]
/// A fast and stable ordered vector of objects with a corresponding string key
/// stored in a hashmap
///
/// Used in TUI to hold items grabbed from a hashmap / hashset for a constant
/// ordering when iterated through and holding historic data
///
/// This should likely be moved to common after unit testing is made later down
/// the line (knowing how that goes, if ever)
pub struct StringLookupVector<T: Clone> {
  lookup: HashMap<String, usize>,
  vector: Vec<NamedValue<T>>,
}

pub struct StringLookupVectorIter<'a, T: Clone> {
  reference: &'a StringLookupVector<T>,
  index: usize,
}

impl<'a, T: Clone> Iterator for StringLookupVectorIter<'a, T> {
  type Item = &'a NamedValue<T>;

  // next() is the only required method
  fn next(&mut self) -> Option<Self::Item> {
    // Check to see if we've finished counting or not.
    let out: Option<Self::Item> = if self.index < self.reference.vector.len() {
      Some(self.reference.vector.get(self.index).unwrap())
    } else {
      None
    };

    // Increment the index
    self.index += 1;

    out
  }
}

#[allow(dead_code)]
impl<T: Clone> StringLookupVector<T> {
  const DEFAULT_CAPACITY: usize = 8;
  pub fn len(&self) -> usize {
    self.vector.len()
  }
  /// Creates a new StringLookupVector with a specified capacity
  pub fn with_capacity(capacity: usize) -> StringLookupVector<T> {
    StringLookupVector {
      lookup: HashMap::<String, usize>::with_capacity(capacity),
      vector: Vec::<NamedValue<T>>::with_capacity(capacity),
    }
  }
  /// Creates a new StringLookupVector with default capacity
  pub fn new() -> StringLookupVector<T> {
    StringLookupVector::with_capacity(StringLookupVector::<T>::DEFAULT_CAPACITY)
  }
  /// Checks if a key is contained within the StringLookupVector
  pub fn contains_key(&self, key: &String) -> bool {
    self.lookup.contains_key(key)
  }

  /// Returns the index of a key in the vector
  pub fn index_of(&self, key: &String) -> Option<usize> {
    self.lookup.get(key).copied()
  }

  /// Returns true if the object was added, and false if it was replaced
  pub fn add(&mut self, name: &String, value: T) {
    if self.contains_key(name) {
      self.vector[self.lookup[name]].value = value;
      return;
    }
    self.lookup.insert(name.clone(), self.vector.len());
    self.vector.push(NamedValue::new(name.clone(), value));
  }

  pub fn remove(&mut self, key: &String) {
    if self.contains_key(key) {
      self.vector.remove(self.lookup[key]);
      self.lookup.remove(key);
    }
  }

  /// Sorts the backing vector by name, making iterations through this 
  /// structure be in alphabetical order
  pub fn sort_by_name(&mut self) {
    self.vector.sort_unstable_by_key(|x| x.name.to_string());
    for i in 0..self.vector.len() {
      *self.lookup.get_mut(&self.vector[i].name).expect("Key has to exist") = i; 
    }
  }

  /// Gets a mutable reference to the item with the given key.
  /// Panics if the key is not valid
  pub fn get(&mut self, key: &String) -> Option<&NamedValue<T>> {
    let index = self.lookup.get(key);
    match index {
      Some(x) => self.vector.get(*x),
      None => None,
    }
  }

  /// Gets a mutable reference to the item with the given index in the vector.
  /// Panics if the key is not valid
  pub fn get_from_index(&mut self, index: usize) -> Option<&NamedValue<T>> {
    self.vector.get(index)
  }

  /// Gets a mutable reference to the item with the given key.
  /// Panics if the key is not valid
  pub fn get_mut(&mut self, key: &String) -> Option<&mut NamedValue<T>> {
    let index = self.lookup.get(key);
    match index {
      Some(x) => self.vector.get_mut(*x),
      None => None,
    }
  }
  
  /// Gets a mutable reference to the item with the given index in the vector.
  /// Panics if the key is not valid
  pub fn get_mut_from_index(
    &mut self,
    index: usize,
  ) -> Option<&mut NamedValue<T>> {
    self.vector.get_mut(index)
  }

  pub fn iter(&self) -> StringLookupVectorIter<T> {
    StringLookupVectorIter::<T> {
      reference: self,
      index: 0,
    }
  }
}

#[derive(Clone)]
pub struct FullValveDatapoint {
  pub voltage: f64,
  pub current: f64,
  pub knows_voltage: bool,
  pub knows_current: bool,
  pub rolling_voltage_average: f64,
  pub rolling_current_average: f64,
  pub state: CompositeValveState,
}

#[derive(Clone)]
pub struct SensorDatapoint {
  pub measurement: Measurement,
  pub rolling_average: f64,
}

#[allow(dead_code)]
impl SensorDatapoint {
  pub fn new(first_measurement: &Measurement) -> SensorDatapoint {
    SensorDatapoint {
      measurement: first_measurement.clone(),
      rolling_average: first_measurement.value,
    }
  }
}

#[derive(Clone)]
pub struct SystemDatapoint {
  pub cpu_usage: f32,
  pub mem_usage: f32,
}

#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum SequenceSendResults {
  NoResult,

  // Do not require response from flight computer
  FailedSend,
  Sent,

  // Require response from flight computer
  ExecutionError,
  ExecutionComplete, // Final stage of success
}

#[derive(Clone)]
pub struct ExecutedCommandStruct {
  pub script: String,
  pub id: u64,
  pub result: SequenceSendResults,
}

pub struct TuiData {
  // Actual tracked
  pub sensors: StringLookupVector<SensorDatapoint>,
  pub valves: StringLookupVector<FullValveDatapoint>,
  pub system_data: StringLookupVector<SystemDatapoint>,

  // Log tracking
  pub filtered_logs: Vec<Log>,
  pub last_log_count: usize,

  /// What "mode" (tab / subpage) the TUI is on
  pub mode: Modes,

  /// The state of the internal console for the TUI (forwards python commands
  /// to flight)
  pub console_state: TUIConsoleState,

  /// The counter for the number of frames that the TUI has drawn
  ///
  /// Used for animations
  pub frame: usize,

  /// Input buffer for console used to input manual commands
  pub console_input: Input,

  // Command history is for arrow key autofill tracking of commands. TODO :
  // MAKE VECDEQUE
  pub command_history: Vec<String>,
  pub command_history_selected: Option<usize>,

  // Log of commands sent
  pub command_log: VecDeque<ExecutedCommandStruct>, /* Command log is for
                                                     * tracking actually
                                                     * send commands for
                                                     * logging */

  // Curr sequence being sent (from command line only)
  pub curr_sequence: Option<tokio::task::JoinHandle<Result<(), ServerError>>>,
  pub curr_sequence_id: u64,

  // Queue of sequences that have not been sent (from command line only)
  pub sequence_queue: VecDeque<Sequence>,

  // Temp / debug variable that says if flight computer is connected
  pub is_connected: bool,

  pub debug_durations: Vec<NamedValue<Duration>>,
  pub last_debug_durations: Vec<NamedValue<Duration>>,
}

impl TuiData {
  pub fn new() -> TuiData {
    TuiData {
      sensors: StringLookupVector::<SensorDatapoint>::new(),
      valves: StringLookupVector::<FullValveDatapoint>::new(),
      system_data: StringLookupVector::<SystemDatapoint>::new(),

      filtered_logs: Vec::<Log>::new(),
      last_log_count: 0,

      mode: Modes::Home,
      console_state: TUIConsoleState::Hidden,

      frame: 0,

      console_input: Input::default(),

      command_history: Vec::<String>::new(),
      command_history_selected: None,

      command_log: VecDeque::<ExecutedCommandStruct>::new(),

      curr_sequence: None,
      curr_sequence_id: 0,

      sequence_queue: VecDeque::<Sequence>::new(),

      is_connected: false,

      debug_durations: Vec::<NamedValue<Duration>>::new(),
      last_debug_durations: Vec::<NamedValue<Duration>>::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn generate_standard_output() -> Vec<NamedValue<u32>> {
    let out : [NamedValue<u32>; 20] = core::array::from_fn(|i| {
      let value = i as u32;

      let name : String = String::from(
        char::from_u32(('a' as u32) + value)
          .expect("it shouldn't be possible to go out of the range when we only
            can go up 6 steps in ASCII")
      ).repeat(i + 1);

      NamedValue::<u32>::new(name, value)
    });
    Vec::<NamedValue<u32>>::from(out)
  }

  #[test]
  fn slv_insertion() {
    let mut expected : Vec<NamedValue<u32>> = Vec::new();
    let mut output : Vec::<NamedValue<u32>>;
    let mut slv : StringLookupVector<u32> = StringLookupVector::new();
    
    output = slv.iter().cloned().collect();
    assert_eq!(output, expected);

    // insert and validate
    for i in 0..6 {
      let value = i as u32;

      let name : String = String::from(
        char::from_u32(('z' as u32) - value)
          .expect("it shouldn't be possible to go out of the range when we only
          can go down 6 steps in ASCII")
      ).repeat(i + 1);

      // It shouldn't have the key before
      assert!(!slv.contains_key(&name));

      slv.add(&name, value);
      expected.push(NamedValue::<u32>::new(name.clone(), value));   
      
      // It should have the key after
      assert!(slv.contains_key(&name));

      output = slv.iter().cloned().collect();
      assert_eq!(output, expected);
    }

    let output : Vec::<NamedValue<u32>> = slv.iter().cloned().collect();

    assert_eq!(output, expected);
  }

  
  #[test]
  fn slv_removal() {
    let base : Vec<NamedValue<u32>> = generate_standard_output();
    let mut output : Vec::<NamedValue<u32>>;
    let mut slv_base : StringLookupVector<u32> = StringLookupVector::new();

    for item in base.clone() {
      slv_base.add(&item.name, item.value);
    }

    // attempt removing every item
    for (index, item) in base.clone().iter().enumerate() {
      let mut expected = base.clone();
      let mut slv = slv_base.clone();
      
      // It should have the key before removal
      assert!(slv.contains_key(&item.name));

      expected.remove(index);
      slv.remove(&item.name);

      // It shouldn't have the key after
      assert!(!slv.contains_key(&item.name));
      
      // Outputs should be the same as if it wasn't added
      output = slv.iter().cloned().collect();
      assert_eq!(output, expected);
    }
  }

  #[test]
  fn slv_sort() {
    let expected : Vec::<NamedValue<u32>> = generate_standard_output();

    let mut slv : StringLookupVector<u32> = StringLookupVector::new();

    // insert everything reversed
    for item in expected.clone().into_iter().rev() {
      slv.add(&item.name, item.value);
    }

    slv.sort_by_name();

    let output : Vec::<NamedValue<u32>> = slv.iter().cloned().collect();

    assert_eq!(output, expected);
  }
}