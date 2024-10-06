pub use common::comm::{Log, LogCategory, LogType};
use std::path::PathBuf;

/// Ring buffer structure.
///
/// Functions akin to a linked list in that you can endlessly push to it,
/// but it has a fixed length backing array and will simply override older
/// entries when it reaches max length.
///
/// Notably used to store the most recent logs in ram for display on the TUI
#[derive(Debug)]
pub struct RingBuffer<T> {
  buffer: Vec<T>, // likely doesn't *need*
  curr_index: usize, /* the current index of the most recently added element
                   * of the buffer */
}

impl<T: Clone> RingBuffer<T> {
  /// Creates a new ringbuffer structure with a maximum capacity of <capacity>
  /// Minimum capacity of 1 (If capacity is given as 0, it will round up to 1)
  pub fn new(mut capacity: usize) -> RingBuffer<T> {
    // Enforce minimum size
    capacity = if capacity > 0 { capacity } else { 1 };
    // Create RingBuffer
    RingBuffer {
      buffer: Vec::<T>::with_capacity(capacity),
      curr_index: capacity - 1,
    }
  }

  /// Pushes an element into the buffer
  pub fn push(&mut self, value: T) {
    self.curr_index = (self.curr_index + 1) % self.buffer.capacity();

    if self.buffer.len() == self.buffer.capacity() {
      // We need to replace the oldest one
      *self.buffer.get_mut(self.curr_index).expect(
        "Invalid index when pushing to ring buffer. This shouldn't happen!",
      ) = value;
    } else {
      self.buffer.push(value);
    }
  }

  /// Iterates over the buffer from newest to oldest
  pub fn iter(&self) -> RingBufferIter<T> {
    RingBufferIter::<T> {
      reference: self,
      index: 0,
    }
  }

  /// Returns iterator over the most recent <max_index> number of entires in
  /// order of oldest to newest
  pub fn len(&self) -> usize {
    self.buffer.len()
  }

  /// Returns whether the ring buffer is empty
  pub fn is_empty(&self) -> bool {
    self.buffer.is_empty()
  }

  /// returns the capacity of the ring buffer
  pub fn capacity(&self) -> usize {
    self.buffer.capacity()
  }

  /// Returns iterator over the most recent <max_index> number of entires in
  /// order of oldest to newest if max_index > length of buffer or max_index is
  /// None, iterates over the entire buffer
  pub fn rev_iter(&self, max_index: Option<usize>) -> RevRingBufferIter<T> {
    RevRingBufferIter::<T> {
      reference: self,
      rev_index: match max_index {
        Some(x) => {
          if self.len() - 1 < x {
            0
          } else {
            self.len() - 1 - x
          }
        }
        None => 0,
      },
      starting_index: if self.len() < self.capacity()
        || self.curr_index == (self.len() - 1)
      {
        0
      } else {
        self.curr_index + 1
      },
    }
  }
}

/// Iteration structure for iterating through a RingBuffer
pub struct RingBufferIter<'a, T: Clone> {
  reference: &'a RingBuffer<T>,
  index: usize,
}

/// Iteration structure for iterating backwards through a RingBuffer
///
/// Notably, this will actually iterate BACKWARDS through the buffer,
/// resulting in the elements being returned in the order of most recent to
/// oldest (This is why it's manually implemented actually, the ring buffer
/// libraries I saw weren't)
pub struct RevRingBufferIter<'a, T: Clone> {
  reference: &'a RingBuffer<T>,
  rev_index: usize,
  starting_index: usize,
}

impl<'a, T: Clone> Iterator for RingBufferIter<'a, T> {
  type Item = &'a T;

  // next() is the only required method
  fn next(&mut self) -> Option<Self::Item> {
    // Check to see if we've finished counting or not.
    let out: Option<Self::Item> = if self.index < self.reference.buffer.len() {
      let real_index = if self.index > self.reference.curr_index {
        // If index results in wrapping around buffer.
        // This will only ever occur if the buffer has already reached it's max
        // capacity, so we use the capacity function.
        // (potential compiler optimizations?)
        self.reference.buffer.capacity()
          - self.reference.curr_index
          - self.index
      } else {
        // If index does not result in wrapping around buffer.
        self.reference.curr_index - self.index
      };
      Some(
        self.reference.buffer
          .get(real_index)
          .unwrap_or_else(|| panic!("Ringbuffer iterator managed to access invalid index {}. This shouldn't happen!", real_index))
        )
    } else {
      None
    };
    // Increment the index
    self.index += 1;

    out
  }
}

impl<'a, T: Clone> Iterator for RevRingBufferIter<'a, T> {
  type Item = &'a T;

  // next() is the only required method
  fn next(&mut self) -> Option<Self::Item> {
    // Check to see if we've finished counting or not.
    let out: Option<Self::Item> = if self.rev_index
      < self.reference.buffer.len()
    {
      let real_index = if self.starting_index + self.rev_index
        > self.reference.buffer.capacity()
      {
        // If index results in wrapping around buffer.
        // This will only ever occur if the buffer has already reached it's max
        // capacity, so we use the capacity function.
        // (potential compiler optimizations?)
        self.starting_index + self.rev_index - self.reference.buffer.capacity()
      } else {
        // If index does not result in wrapping around buffer.
        self.starting_index + self.rev_index
      };
      Some(
          self.reference.buffer
            .get(real_index)
            .unwrap_or_else(|| panic!("Reverse Ringbuffer iterator managed to access invalid index {}. This shouldn't happen!", real_index))
        )
    } else {
      None
    };
    // Increment the index
    self.rev_index += 1;

    out
  }
}

#[derive(Debug)]
#[allow(dead_code)]
/// The wrapper structure around all logging in servo
///
/// Handles both logging to files and logging on the TUI
pub struct LogsController {
  /// a ram-stored set of the most recent logs to prevent constant read/writes
  log_history: RingBuffer<Log>,
  /// (To be implemented), the file that is logged to
  log_file: PathBuf,
  /// The name of the source (a la "servo" or "flight") / origin of the log
  source_name: String,
  /// The number of logs that have been received
  ///
  /// Used by the TUI to track whether it needs to refilter the log history
  /// among other things
  logs_received: usize,
}

static DEFAULT_LOGS_CONTROLLER_CAPACITY: usize = 1024;
impl LogsController {
  /// Create a log controller with the ram-stored log history capacity of
  /// `capacity`, logging to the file at `file_path`,
  /// with the default source name used by `log_here()`` of `source_name``
  ///
  /// Capacities below 1 will round up to 1
  pub fn with_capacity(
    capacity: usize,
    file_path: PathBuf,
    source_name: String,
  ) -> LogsController {
    LogsController {
      log_history: RingBuffer::<Log>::new(capacity),
      log_file: file_path,
      source_name,
      logs_received: 0,
    }
  }

  /// Create a log controller
  /// logging to the file `file_path`
  /// with the default source name used by log_here() of `source_name`
  pub fn new(file_path: PathBuf, source_name: String) -> LogsController {
    Self::with_capacity(
      DEFAULT_LOGS_CONTROLLER_CAPACITY,
      file_path,
      source_name,
    )
  }

  /// Log a log to file and history
  pub fn log(&mut self, log: Log) {
    self.log_history.push(log);
    self.logs_received += 1;
  }

  /// Log macro for adding a log from an external source at current time
  pub fn log_now(
    &mut self,
    log_type: LogType,
    log_category: LogCategory,
    source: String,
    header: String,
    contents: String,
  ) {
    self.log(Log {
      log_type,
      log_category,
      time_stamp: std::time::SystemTime::now(),
      source,
      header,
      contents,
    })
  }

  /// Log macro for adding a log from default source at current time
  pub fn log_here(
    &mut self,
    log_type: LogType,
    log_category: LogCategory,
    header: String,
    contents: String,
  ) {
    self.log(Log {
      log_type,
      log_category,
      time_stamp: std::time::SystemTime::now(),
      source: self.source_name.clone(),
      header,
      contents,
    })
  }

  /// Iterate over the Logs from newest to oldest
  pub fn iter(&self) -> RingBufferIter<Log> {
    self.log_history.iter()
  }

  /// Iterate over the last `max_index` Logs from oldest to newest
  /// 
  /// max_index is NOT the number of logs.
  /// 
  /// This iterator will iterate over either `max_index + 1` or `self.len()`
  /// logs, whichever is lower.
  pub fn rev_iter(&self, max_index: Option<usize>) -> RevRingBufferIter<Log> {
    self.log_history.rev_iter(max_index)
  }

  /// Determine if logs have been added since the `last_known_log_count`'ths log
  pub fn updated_since(&self, last_known_log_count: usize) -> bool {
    last_known_log_count != self.logs_received
  }

  /// Determine the number of logs added this session
  pub fn log_count(&self) -> usize {
    self.logs_received
  }
}


#[cfg(test)]
mod tests {
  use std::time::SystemTime;
  use super::*;

  fn generate_five_sample_logs() -> [Log; 5] {
    [
      Log {
        log_type : LogType::Debug,
        log_category : LogCategory::Other,
        time_stamp : SystemTime::now(),
        source : String::from("debug_tests_a"),
        header : String::from("This is log 1"),
        contents : String::from("At least I hope so")
      },
      Log {
        log_type : LogType::Standard,
        log_category : LogCategory::Unknown,
        time_stamp : SystemTime::now(),
        source : String::from("debug_tests_b"),
        header : String::from("This is log 2"),
        contents : String::from("if this works")
      },
      Log {
        log_type : LogType::Error,
        log_category : LogCategory::Sensors,
        time_stamp : SystemTime::now(),
        source : String::from("debug_tests_c"),
        header : String::from("This is log 3"),
        contents : String::from("Also the hydrogen tank exploded")
      },
      Log {
        log_type : LogType::Standard,
        log_category : LogCategory::Sequences,
        time_stamp : SystemTime::now(),
        source : String::from("debug_tests_d"),
        header : String::from("This is log 4"),
        contents : String::from("Also the fuel tank exploded")
      },
      Log {
        log_type : LogType::Error,
        log_category : LogCategory::Valves,
        time_stamp : SystemTime::now(),
        source : String::from("debug_tests_e"),
        header : String::from("This is log 5"),
        contents : String::from("Also the oxygen tank exploded")
      }
    ]
  }

  #[test]
  fn logging_controller_basic() {
    let mut controller = LogsController::new(
      PathBuf::from("tests.txt"),
      String::from("debug_tests")
    );

    let logs : [Log; 5] = generate_five_sample_logs();
    for log in logs.iter().cloned() {
      controller.log(log);
    }


    // Ensure normal iteration is as expected
    {
      let mut expected = Vec::from(logs.clone());
      expected.reverse(); // reversed as logs are read newest to oldest

      let output : Vec<Log> = controller.iter().cloned().collect();
      assert_eq!(output, expected);
    }

    
    // Ensure reverse iteration is as expected
    {
      // check reverse iterator of most recent 3 logs
      let expected = Vec::from(logs.clone()).split_off(2); 
      let output : Vec<Log> = controller.rev_iter(Some(2)).cloned().collect();
      assert_eq!(output, expected);
    }
  }
}