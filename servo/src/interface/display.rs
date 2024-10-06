use super::tabs::{home_menu, logs_tab};
use super::*;

use crate::server::{
  error::{internal, ServerError},
  Shared,
};
use std::{
  error::Error,
  io::{self, Stdout},
  mem::take,
  ops::Div,
  sync::atomic::{AtomicU64, Ordering},
  time::{Duration, Instant},
  vec::Vec,
};
use sysinfo::{CpuExt, System, SystemExt};

use common::comm::Sequence;
use std::string::String;

use tokio::{task::JoinHandle, time::sleep};

use crossterm::{
  event::{
    self,
    DisableMouseCapture,
    EnableMouseCapture,
    Event,
    KeyCode,
    KeyEventKind,
    KeyModifiers,
  },
  execute,
  terminal::{
    disable_raw_mode,
    enable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
  },
};
use ratatui::{prelude::*, widgets::*};

use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use unicode_width;

use std::sync::Arc;

/// A atomic (thread safe) object used to give command line sequences unique
/// ID's
///
/// May be replaced by a universal sequence ID'ing system for better logging
static COMMAND_INDEX_ATOMIC: AtomicU64 = AtomicU64::new(0);

/// The maximum length of the command log (The actual display of commands and
/// their results)
const COMMAND_LOG_MAX_LEN: usize = 25;

/// The maximum length of command history (autofill / arrow key stuff)
const COMMAND_HISTORY_MAX_LENGTH: usize = 20;

/// Maximum size of the fildered logs vector
const FILTERED_LOGS_MAX_SIZE: usize = 64;

fn get_selected_tab(mode: Modes) -> usize {
  match mode {
    Modes::Home => 0,
    Modes::Logs => 1,
  }
}

impl TuiData {
  async fn attempt_progress_sequence_queue(&mut self, shared: &Arc<Shared>) {
    // move out of reference
    let mv_curr_sequence: Option<JoinHandle<Result<(), ServerError>>> =
      take(&mut self.curr_sequence);

    // do logic
    match mv_curr_sequence {
      Some(handle) => {
        if handle.is_finished() {
          let result = handle.await;
          let command_res: SequenceSendResults;
          if result.is_ok() {
            if result.unwrap().is_ok() {
              command_res = SequenceSendResults::Sent
            } else {
              command_res = SequenceSendResults::FailedSend
            }
          } else {
            command_res = SequenceSendResults::FailedSend
          }

          {
            let mut logs = shared.logs.0.lock().await;
            logs.log_here(
              if command_res == SequenceSendResults::FailedSend {
                LogType::Error
              } else {
                LogType::Success
              },
              LogCategory::Sequences,
              format!(
                "{} Command {} to flight",
                match command_res {
                  SequenceSendResults::Sent => "Successfully Sent",
                  SequenceSendResults::FailedSend => "Failed to Send",
                  _ => "Unexpected Result for Sending",
                },
                self.curr_sequence_id
              ),
              String::new(),
            );
          }

          for command in &mut self.command_log {
            if command.id == self.curr_sequence_id {
              command.result = command_res;
              break;
            }
          }
          self.curr_sequence = None;
        } else {
          // put it back if it's not done
          self.curr_sequence = Some(handle);
        }
      }
      None => {
        if !self.sequence_queue.is_empty() {
          let sequence = self.sequence_queue.pop_front().unwrap();

          let sequence_id: u64 =
            COMMAND_INDEX_ATOMIC.fetch_add(1, Ordering::SeqCst);
          self.curr_sequence_id = sequence_id;

          {
            let mut logs = shared.logs.0.lock().await;
            logs.log_here(
              LogType::Standard,
              LogCategory::Sequences,
              format!("Sending Command {} to flight", self.curr_sequence_id),
              sequence.script.clone(),
            );
          }
          self.command_log.push_front(ExecutedCommandStruct {
            script: sequence.script.clone(),
            id: sequence_id,
            result: SequenceSendResults::NoResult,
          });
          if self.command_log.len() > COMMAND_LOG_MAX_LEN {
            self.command_log.pop_back();
          }
          self.curr_sequence =
            Some(tokio::spawn(send_sequence(sequence, shared.clone())));
        }
      }
    }
  }

  // WARNING : in it's current state this WILL freeze up anything that attempts
  // to log anything until it is finished This means that if it takes a long
  // time to filter, it will jam up any system that uses logs in series
  async fn update_filtered_logs(&mut self, shared: &Shared) {
    // get logs
    let logs = shared.logs.0.lock().await;
    // if logs are
    if !logs.updated_since(self.last_log_count) {
      return;
    }
    self.last_log_count = logs.log_count();
    self.filtered_logs.clear();
    for log in logs.rev_iter(Some(FILTERED_LOGS_MAX_SIZE)) {
      // you'd do filtering here but I'm just not gonna rn
      self.filtered_logs.push(log.clone());

      // if past how many are allowed to be here at once, return
      if self.filtered_logs.len() > FILTERED_LOGS_MAX_SIZE {
        break;
      }
    }
  }
}

/// Updates the backing tui_data instance that is used in the rendering
/// functions
async fn update_information(
  tui_data: &mut TuiData,
  shared: &Shared,
  system: &mut System,
) {
  // display system statistics
  system.refresh_cpu();
  system.refresh_memory();

  let hostname = system
    .host_name()
    .unwrap_or("\x1b[33mnone\x1b[0m".to_owned());

  if !tui_data.system_data.contains_key(&hostname) {
    tui_data.system_data.add(
      &hostname,
      SystemDatapoint {
        cpu_usage: 0.0,
        mem_usage: 0.0,
      },
    );
  }

  let servo_usage: &mut SystemDatapoint = &mut tui_data
    .system_data
    .get_mut(&hostname)
    .expect("Already checked before. so this should never be invalid")
    .value;

  servo_usage.cpu_usage = system
    .cpus()
    .iter()
    .fold(0.0, |util, cpu| util + cpu.cpu_usage())
    .div(system.cpus().len() as f32);

  servo_usage.mem_usage =
    system.used_memory() as f32 / system.total_memory() as f32 * 100.0;

  tui_data.update_filtered_logs(shared).await;

  // display sensor data
  let vehicle_state = shared.vehicle.0.lock().await.clone();

  let sensor_readings =
    vehicle_state.sensor_readings.iter().collect::<Vec<_>>();

  let valve_states = vehicle_state.valve_states.iter().collect::<Vec<_>>();

  let mut sort_needed = false;
  for (name, value) in valve_states {
    match tui_data.valves.get_mut(name) {
      Some(x) => x.value.state = value.clone(),
      None => {
        tui_data.valves.add(
          name,
          FullValveDatapoint {
            voltage: 0.0,
            current: 0.0,
            knows_voltage: false,
            knows_current: false,
            rolling_voltage_average: 0.0,
            rolling_current_average: 0.0,
            state: value.clone(),
          },
        );
        sort_needed = true;
      }
    }
  }
  if sort_needed {
    tui_data.valves.sort_by_name();
  }
  const CURRENT_SUFFIX: &str = "_I";
  const VOLTAGE_SUFFIX: &str = "_V";
  sort_needed = false;
  for (name, value) in sensor_readings {
    if name.len() > 2 {
      if name.ends_with(CURRENT_SUFFIX) {
        let mut real_name = name.clone();
        let _ = real_name.split_off(real_name.len() - 2);
        if let Some(valve_datapoint) = tui_data.valves.get_mut(&real_name) {
          valve_datapoint.value.current = value.value;
          if !valve_datapoint.value.knows_current {
            valve_datapoint.value.rolling_current_average = value.value;
            valve_datapoint.value.knows_current = true;
          } else {
            valve_datapoint.value.rolling_current_average *= 0.8;
            valve_datapoint.value.rolling_current_average += 0.2 * value.value;
          }
          continue;
        }
      } else if name.ends_with(VOLTAGE_SUFFIX) {
        let mut real_name = name.clone();
        let _ = real_name.split_off(real_name.len() - 2);
        if let Some(valve_datapoint) = tui_data.valves.get_mut(&real_name) {
          valve_datapoint.value.voltage = value.value;
          if !valve_datapoint.value.knows_voltage {
            valve_datapoint.value.rolling_voltage_average = value.value;
            valve_datapoint.value.knows_voltage = true;
          } else {
            valve_datapoint.value.rolling_voltage_average *= 0.8;
            valve_datapoint.value.rolling_voltage_average += 0.2 * value.value;
          }
          continue;
        }
      }
    }
    match tui_data.sensors.get_mut(name) {
      Some(x) => {
        x.value.measurement = value.clone();
        x.value.rolling_average *= 0.8;
        x.value.rolling_average += 0.2 * value.value;
      }
      None => {
        tui_data.sensors.add(
          name,
          SensorDatapoint {
            measurement: value.clone(),
            rolling_average: value.value,
          },
        );
        sort_needed = true;
      }
    }
  }
  if sort_needed {
    tui_data.sensors.sort_by_name();
  }
}

fn handle_key_console(key: crossterm::event::KeyEvent, tui_data: &mut TuiData) {
  // One press only commands
  if key.kind == KeyEventKind::Press {
    match key.code {
      KeyCode::Enter => {
        // If currently looking at history, take that instead
        if let Some(x) = tui_data.command_history_selected {
          tui_data.console_input =
            Input::new(tui_data.command_history[x].clone());
          tui_data.command_history_selected = None;
        }

        let input = tui_data.console_input.value();

        if !input.is_empty() {
          // Send data to flight in async method later (queue it up)
          let sequence: Sequence = Sequence {
            name: String::from("manual"),
            script: String::from(input),
          };
          tui_data.sequence_queue.push_back(sequence);

          // Save to history if not a dup of last
          if tui_data.command_history.is_empty()
            || tui_data.command_history[0] != input
          {
            // Not worth optimizing
            tui_data.command_history.insert(0, String::from(input));
            while tui_data.command_history.len() > COMMAND_HISTORY_MAX_LENGTH {
              tui_data.command_history.pop();
            }
          }
          tui_data.console_input.reset();
        }
      }
      KeyCode::Up => match tui_data.command_history_selected {
        Some(x) => {
          if x + 1 < tui_data.command_history.len() {
            tui_data.command_history_selected = Some(x + 1);
          }
        }
        None => {
          if !tui_data.command_history.is_empty() {
            tui_data.command_history_selected = Some(0);
          }
        }
      },
      KeyCode::Down => {
        if let Some(x) = tui_data.command_history_selected {
          if x > 0 {
            tui_data.command_history_selected = Some(x - 1);
          } else {
            tui_data.command_history_selected = None;
          }
        }
      }
      KeyCode::Esc => tui_data.console_state = TUIConsoleState::Hidden,
      _ => {
        // If currently looking at history, use that instead
        if let Some(x) = tui_data.command_history_selected {
          tui_data.console_input =
            Input::new(tui_data.command_history[x].clone());
          tui_data.command_history_selected = None;
        }
        tui_data.console_input.handle_event(&Event::Key(key));
      }
    }
  } else {
    tui_data.console_input.handle_event(&Event::Key(key));
  }
}

/// A function called every display round that draws the ui and handles user
/// input removed from display due to certain functions returning generic
/// errors, which cause the serializer to have an aneurysm and thus not work
/// with async.
fn display_round(
  terminal: &mut Terminal<CrosstermBackend<Stdout>>,
  tui_data: &mut TuiData,
  tick_rate: Duration,
  last_tick: &mut Instant,
) -> bool {
  // Increment the frame
  tui_data.frame += 1;

  // Draw the TUI
  let _ = terminal.draw(|f| servo_ui(f, tui_data));

  // Handle user input
  loop {
    // check for inputs
    match crossterm::event::poll(Duration::from_millis(0)) {
      // If there is no input waiting to be handled, quit input handling loop
      Ok(x) => {
        if !x {
          break;
        }
      }
      // If there is an error, log it
      // TODO : Actually log this error instead of printing
      Err(err) => {
        println!("Input polling failed! : ");
        println!("{}", err);
        return false;
      }
    };

    // Read the input
    let read_res = event::read();

    // If reading failed, big sad, print it
    if read_res.is_err() {
      println!("Input reading failed : ");
      println!("{}", read_res.unwrap_err());
      return false;
    }
    // If a quit command is recieved, return false to signal to quit
    if let Event::Key(key) = read_res.unwrap() {
      // We don't care about anything but key presses
      if key.kind != KeyEventKind::Press {
        continue;
      }
      match key.code {
        KeyCode::Char('c') => {
          if key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
          } else if tui_data.console_state != TUIConsoleState::Hidden {
            handle_key_console(key, tui_data);
          }
        }
        KeyCode::Char('C') => {
          if key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
          } else if tui_data.console_state != TUIConsoleState::Hidden {
            handle_key_console(key, tui_data);
          }
        }
        KeyCode::Tab => {
          tui_data.mode = match tui_data.mode {
            Modes::Home => Modes::Logs,
            Modes::Logs => Modes::Home,
          }
        }
        KeyCode::Char('`') => match tui_data.console_state {
          TUIConsoleState::Hidden => {
            tui_data.console_state = TUIConsoleState::Flight
          }
          _ => handle_key_console(key, tui_data),
        },
        KeyCode::Char('~') => match tui_data.console_state {
          TUIConsoleState::Hidden => {
            tui_data.console_state = TUIConsoleState::Flight
          }
          _ => handle_key_console(key, tui_data),
        },
        _ => {
          if tui_data.console_state != TUIConsoleState::Hidden {
            handle_key_console(key, tui_data);
          }
        }
      }
    }
  }

  //
  if last_tick.elapsed() >= tick_rate {
    last_tick.clone_from(&Instant::now());
  }

  // If no quit command is recieved, return false to signal to continue
  true
}

/// Attempts to restore the terminal to the pre-servo TUI state
fn restore_terminal(
  terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
  // restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  //if let Err(err) = res {
  //    println!("{err:?}");
  //}

  Ok(())
}

// TODO : maybe this shouldn't block the entire time?
async fn send_sequence(
  sequence: Sequence,
  shared: Arc<Shared>,
) -> Result<(), ServerError> {
  if let Some(flight) = shared.flight.0.lock().await.as_mut() {
    // Send the sequence to the flight computer
    flight.send_sequence(sequence).await.map_err(internal)?;
    Ok(())
  } else {
    Err(internal("flight computer not connected"))
  }
}

/// The async function that drives the entire TUI.
/// Returns once it is manually quit (from within display_round)
pub async fn display(shared: Arc<Shared>) -> io::Result<()> {
  // setup terminal
  enable_raw_mode()?;

  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut system = System::new_all();

  // The minimum duration between the start of each tui tick
  let tick_rate = Duration::from_millis(25);

  // The data structure that holds almost all information on the tui
  let mut tui_data: TuiData = TuiData::new();

  // Time of last GUI tick
  let mut last_tick = Instant::now();

  // How many GUI ticks should pass between each update information from the
  // flight computer (used to keep FPS high for the terminal)
  let update_rate = 4;
  let mut update_tick = 0;

  loop {
    // Duration Tracking Code
    let update_start_time = Instant::now();

    // Update last_debug_durations and clear debug_durations to be filled this
    // cycle
    (tui_data.last_debug_durations, tui_data.debug_durations) =
      (tui_data.debug_durations, tui_data.last_debug_durations);
    tui_data.debug_durations.clear();

    tui_data.is_connected =
      if let Some(flight) = shared.flight.0.lock().await.as_mut() {
        // Send the sequence to the flight computer
        !flight.check_closed()
      } else {
        false
      };

    if update_tick == 0 {
      update_information(&mut tui_data, &shared, &mut system).await;
    }
    update_tick += 1;
    update_tick %= update_rate;

    // Draw the TUI and handle user input, return if told to.
    if !display_round(&mut terminal, &mut tui_data, tick_rate, &mut last_tick) {
      break;
    }
    // Handle any sequences that need sending
    tui_data.attempt_progress_sequence_queue(&shared).await;

    // Determine how long everything took

    let total_duration = update_start_time.elapsed();

    tui_data.debug_durations.push(NamedValue::<Duration>::new(
      String::from("Total"),
      total_duration,
    ));

    // Wait until next tick
    if total_duration < tick_rate {
      sleep(tick_rate - total_duration).await;
    }
  }

  // Attempt to restore terminal
  let res = restore_terminal(&mut terminal);
  if let Err(err) = res {
    return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
  }

  Ok(())
}

/// Basic overhead ui drawing function.
/// Creates the main overarching tab and then draws the selected tab in the
/// remaining space
fn servo_ui(f: &mut Frame, tui_data: &TuiData) {
  // Vertically chunk the TUI into rendered regions
  let vertical_sections: std::rc::Rc<[Rect]> = Layout::default()
    .direction(Direction::Vertical)
    .constraints(match tui_data.console_state {
      // chunked as [tabs / debug time, TUI display, internal console (if
      // applicable)]
      TUIConsoleState::Hidden => [
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(0),
      ],
      TUIConsoleState::Flight => [
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
      ],
    })
    .split(f.size());

  upper_tui_section(f, vertical_sections[0], tui_data);

  match tui_data.mode {
    Modes::Home => home_menu(f, vertical_sections[1], tui_data),
    Modes::Logs => logs_tab(f, vertical_sections[1], tui_data),
    _ => draw_empty(f, vertical_sections[1]),
  };

  if tui_data.console_state != TUIConsoleState::Hidden {
    // Draw the console
    let console_area = draw_sub_console(f, vertical_sections[2], tui_data);

    // 2 of the width is for borders and 1 is for cursor
    let width = console_area.width.max(3) - 3;
    let scroll = tui_data.console_input.visual_scroll(width as usize);

    let cursor = match tui_data.command_history_selected {
      Some(x) => unicode_width::UnicodeWidthStr::width(
        tui_data.command_history[x]
          .get(0..tui_data.command_history[x].len())
          .unwrap(),
      ),
      None => tui_data.console_input.visual_cursor(),
    };

    // Put cursor inside of command line to communicate editting
    //    Currently flashes wildly as it will turn on whenever the TUI updates
    //    very low priority change to fix it though
    f.set_cursor(
      // Put cursor past the end of the input text
      console_area.x + ((cursor).max(scroll) - scroll) as u16 + 1,
      // Move one line down, from the border to the input line
      console_area.y + 1,
    );
  }
}

/// Renders the top section of the tui containing tabs and any debug
/// information.
fn upper_tui_section(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  // Make room for displaying debug durations
  let mut upper_constraints = vec![Constraint::Fill(1)];
  upper_constraints.extend(
    [Constraint::Length(10)].repeat(tui_data.last_debug_durations.len()),
  );

  let upper_tab: std::rc::Rc<[Rect]> = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(upper_constraints)
    .split(area);

  // Padding is written into the actual strings here instead of using the built
  // in version in Tabs, as Tabs' padding does not take on the style of the
  // text within, Which we use to indicate selection
  let tab_menu = Tabs::new(vec![" Home ", " Logs ", " Unused "])
    .block(Block::default().title("Tabs").borders(Borders::ALL))
    .style(YJSP_STYLE)
    .highlight_style(YJSP_STYLE.bg(YJSP_YELLOW).fg(BLACK).bold())
    .select(get_selected_tab(tui_data.mode))
    .padding("", "")
    .divider(symbols::line::VERTICAL);

  f.render_widget(tab_menu, upper_tab[0]);

  for (index, debug_duration) in
    tui_data.last_debug_durations.iter().enumerate()
  {
    let duration_millis = debug_duration.value.as_micros() as f32 / 1000.0;

    let last_update_sector = Paragraph::new(
      Line::from(format!("{:.1} ms", duration_millis)).right_aligned(),
    )
    .block(
      Block::default()
        .title(debug_duration.name.clone())
        .borders(Borders::ALL),
    )
    .style(YJSP_STYLE);

    f.render_widget(last_update_sector, upper_tab[index + 1]);
  }
}

fn draw_sub_console(f: &mut Frame, area: Rect, tui_data: &TuiData) -> Rect {
  let border = Block::default()
    .style(YJSP_STYLE)
    .title("Console")
    .borders(Borders::ALL);

  let horizontal = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Fill(1),
      Constraint::Length(40 + 75 + 45),
      Constraint::Fill(1),
    ]) // Fill to the identical width of the other stuff
    .split(area);

  // Filler for left side of screen to center actual data
  draw_empty(f, horizontal[0]);
  // Filler for right side of screen to center actual data
  draw_empty(f, horizontal[2]);

  let console_area = &horizontal[1];

  // 2 of the width is for borders and 1 is for cursor
  let width = console_area.width.max(3) - 3;
  let scroll = tui_data.console_input.visual_scroll(width as usize);

  let text: String = match tui_data.command_history_selected {
    Some(x) => tui_data.command_history[x].clone(),
    None => String::from(tui_data.console_input.value()),
  };

  let console_display = Paragraph::new(text)
    .style(YJSP_STYLE.fg(WHITE))
    .scroll((0, scroll as u16))
    .block(border);

  f.render_widget(console_display, *console_area);

  *console_area
}
