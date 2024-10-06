use super::*;

use std::vec::Vec;

use ratatui::{prelude::*, widgets::*};
use std::string::String;

/// Maybe these should be moved to generic if they get used elsewhere?
fn get_state_style(state: ValveState) -> Style {
  match state {
    ValveState::Undetermined => YJSP_STYLE.fg(WHITE).bg(DARK_GREY).bold(),
    ValveState::Disconnected => YJSP_STYLE.fg(BLACK).bg(GREY).bold(),
    ValveState::Open => YJSP_STYLE.fg(BLACK).bg(DESATURATED_GREEN).bold(),
    ValveState::Closed => YJSP_STYLE.fg(BLACK).bg(DESATURATED_RED).bold(),
    ValveState::Fault => YJSP_STYLE.fg(BLACK).bg(DESATURATED_BLUE).bold(),
    _ => YJSP_STYLE.fg(BLACK).bg(UNKNOWN_PURPLE).bold(),
  }
}

fn get_full_row_style(state: ValveState) -> Style {
  match state {
    ValveState::Undetermined => YJSP_STYLE.fg(WHITE).bg(DARK_GREY),
    ValveState::Disconnected => YJSP_STYLE.fg(BLACK).bg(GREY),
    ValveState::Fault => YJSP_STYLE.fg(BLACK).bg(DESATURATED_RED),
    _ => YJSP_STYLE.fg(WHITE),
  }
}

fn get_valve_name_style(state: ValveState) -> Style {
  match state {
    ValveState::Undetermined => YJSP_STYLE.bg(DARK_GREY).bold(),
    ValveState::Disconnected => YJSP_STYLE.fg(BLACK).bg(GREY).bold(),
    ValveState::Fault => YJSP_STYLE.bg(DESATURATED_RED).bold(),
    _ => YJSP_STYLE.bold(),
  }
}

/// Home tab render function displaying
/// System, Valves, and Sensor Information
pub fn home_menu(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  let horizontal = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Fill(1),
      Constraint::Length(40),
      Constraint::Length(75),
      Constraint::Length(45),
      Constraint::Fill(1),
    ])
    .split(area);

  draw_empty(f, horizontal[0]); // Ride side filler

  draw_system_info(f, horizontal[1], tui_data); // System Info Column

  draw_valves(f, horizontal[2], tui_data); // Valve Data Column

  draw_sensors(f, horizontal[3], tui_data); // Sensor Data Column

  draw_empty(f, horizontal[4]); // Left side filler
}

/// Draws system info as listed in tui_data.system_data
/// See update_information for how this data is gathered
fn draw_system_info(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  let all_systems: &StringLookupVector<SystemDatapoint> = &tui_data.system_data;

  // Styles used in table
  let name_style = YJSP_STYLE.bold();
  let data_style = YJSP_STYLE.fg(WHITE);

  // Make rows
  // with_capacity intentionally overshoots the content of this section
  // to avoid any chance of a realloc.
  // After all, memory is "free"
  let mut rows: Vec<Row> = Vec::<Row>::with_capacity(all_systems.len() * 4 + 5);

  for name_datapoint_pair in all_systems.iter() {
    let name: &String = &name_datapoint_pair.name;
    let datapoint: &SystemDatapoint = &name_datapoint_pair.value;

    // Name of system
    rows.push(
      Row::new(vec![
        Cell::from(Span::from(name.clone()).into_centered_line()),
        Cell::from(Span::from("")),
        Cell::from(Span::from("")),
      ])
      .style(name_style),
    );

    //  CPU Usage
    rows.push(
      Row::new(vec![
        Cell::from(Span::from("CPU Usage").into_right_aligned_line()),
        Cell::from(
          Span::from(format!("{:.1}", datapoint.cpu_usage))
            .into_right_aligned_line(),
        ),
        Cell::from(Span::from("%")),
      ])
      .style(data_style),
    );

    //  Memory Usage
    rows.push(
      Row::new(vec![
        Cell::from(Span::from("Memory Usage").into_right_aligned_line()),
        Cell::from(
          Span::from(format!("{:.1}", datapoint.mem_usage))
            .into_right_aligned_line(),
        ),
        Cell::from(Span::from("%")),
      ])
      .style(data_style),
    );
  }

  // Temp flight computer debug stuff

  rows.push(
    Row::new(vec![
      Cell::from(Span::from("Flight Computer").into_centered_line()),
      Cell::from(Span::from("")),
      Cell::from(Span::from("")),
    ])
    .style(name_style),
  );
  rows.push(
    Row::new(vec![
      Cell::from(Span::from("Is Connected").into_right_aligned_line()),
      Cell::from(
        Span::from(if tui_data.is_connected { "Yes" } else { "No" })
          .into_right_aligned_line(),
      ),
      Cell::from(Span::from("")),
    ])
    .style(data_style),
  );

  //  ~Fixed size widths that can scale to a smaller window
  let widths = [Constraint::Max(20), Constraint::Max(12), Constraint::Max(2)];

  //  Make the table itself
  let system_table: Table<'_> = Table::new(rows, widths)
    .style(name_style)
    // It has an optional header, which is simply a Row always visible at the
    // top.
    .header(
      Row::new(vec![
        Span::from("Name").into_centered_line(),
        Span::from("Value").into_centered_line(),
        Line::from(""),
      ])
      .style(Style::new().bold())
      // To add space between the header and the rest of the rows, specify the
      // margin
      .bottom_margin(1),
    )
    // As any other widget, a Table can be wrapped in a Block.
    .block(Block::default().title("Systems").borders(Borders::ALL))
    // The selected row and its content can also be styled.
    .highlight_style(Style::new().reversed())
    // ...and potentially show a symbol in front of the selection.
    .highlight_symbol(">>");

  //  Render
  f.render_widget(system_table, area);
}

/// Draws valve states as listed in tui_data.valves
/// See update_information for how this data is gathered
fn draw_valves(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  //  Get valve states from TUI
  let full_valves: &StringLookupVector<FullValveDatapoint> = &tui_data.valves;

  // Make rows
  let mut rows: Vec<Row> = Vec::<Row>::with_capacity(full_valves.len());
  for pair in full_valves.iter() {
    let name = &pair.name;
    let datapoint = &pair.value;

    //  Get base style used in this row based on the actual (derived) state of
    // the valve
    let normal_style = get_full_row_style(datapoint.state.actual);
    let name_style = get_valve_name_style(datapoint.state.actual);

    // Determine rolling change of voltage and current via value - rolling
    // average of value as calculated by update_information And color code
    // the change based on it's magnitude and sign (increasing / decreasing)
    // Color coding is based on fixed thresholds set for voltage and current
    // independently
    let d_v = datapoint.voltage - datapoint.rolling_voltage_average;
    let d_v_style: Style;
    if d_v.abs() < 0.1 {
      d_v_style = normal_style;
    } else if d_v > 0.0 {
      d_v_style = normal_style.fg(Color::Green);
    } else {
      d_v_style = normal_style.fg(Color::Red);
    }

    let d_i: f64 = datapoint.current - datapoint.rolling_current_average;
    let d_i_style: Style;
    if d_i.abs() < 0.025 {
      d_i_style = normal_style;
    } else if d_i > 0.0 {
      d_i_style = normal_style.fg(Color::Green);
    } else {
      d_i_style = normal_style.fg(Color::Red);
    }

    let voltage_rows: [Cell; 2] = if datapoint.knows_voltage {
      [
        Cell::from(
          Span::from(format!("{:.2}", datapoint.voltage))
            .into_right_aligned_line(),
        ), // Voltage
        Cell::from(
          Span::from(format!("{:+.3}", d_v)).into_right_aligned_line(),
        )
        .style(d_v_style), // Rolling change of voltage
      ]
    } else {
      [Cell::from(""), Cell::from("")]
    };

    let current_rows: [Cell; 2] = if datapoint.knows_current {
      [
        Cell::from(
          Span::from(format!("{:.3}", datapoint.current))
            .into_right_aligned_line(),
        ), // Current
        Cell::from(
          Span::from(format!("{:+.3}", d_i)).into_right_aligned_line(),
        )
        .style(d_i_style), // Rolling change of current
      ]
    } else {
      [Cell::from(""), Cell::from("")]
    };

    // Make the actual row of info
    rows.push(
      Row::new(vec![
        Cell::from(
          Span::from(name.clone())
            .into_centered_line()
            .style(name_style),
        ), // Name of Valve
        voltage_rows[0].clone(),
        voltage_rows[1].clone(),
        current_rows[0].clone(),
        current_rows[1].clone(),
        Cell::from(
          // Actual / Derived state of valve
          Span::from(format!("{}", datapoint.state.actual))
            .into_centered_line(),
        )
        .style(get_state_style(datapoint.state.actual)),
        Cell::from(
          // Commanded state of valve
          Span::from(format!("{}", datapoint.state.commanded))
            .into_centered_line(),
        )
        .style(get_state_style(datapoint.state.commanded)),
      ])
      .style(normal_style),
    );
  }

  let widths = [
    Constraint::Length(12),
    Constraint::Length(7),
    Constraint::Length(8),
    Constraint::Length(8),
    Constraint::Length(9),
    Constraint::Length(12),
    Constraint::Length(12),
  ];

  let valve_table: Table<'_> = Table::new(rows, widths)
    .style(YJSP_STYLE)
    // It has an optional header, which is simply a Row always visible at the
    // top.
    .header(
      Row::new(vec![
        Span::from("Name").into_centered_line(),
        Span::from("Voltage").into_right_aligned_line(),
        Line::from(""),
        Span::from("Current").into_right_aligned_line(),
        Line::from(""),
        Span::from("Derived").into_centered_line(),
        Span::from("Commanded").into_centered_line(),
      ])
      .style(Style::new().bold())
      // To add space between the header and the rest of the rows, specify the
      // margin
      .bottom_margin(1),
    )
    // As any other widget, a Table can be wrapped in a Block.
    .block(Block::default().title("Valves").borders(Borders::ALL))
    // The selected row and its content can also be styled.
    .highlight_style(Style::new().reversed())
    // ...and potentially show a symbol in front of the selection.
    .highlight_symbol(">>");

  f.render_widget(valve_table, area);
}

/// Draws sensors as listed in tui_data.sensors
/// See update_information for how this data is gathered
fn draw_sensors(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  //  Get sensor measurements from TUI
  let full_sensors: &StringLookupVector<SensorDatapoint> = &tui_data.sensors;

  //  Styles used in table
  let normal_style = YJSP_STYLE;
  let data_style = normal_style.fg(WHITE);

  //  Make rows
  let mut rows: Vec<Row> = Vec::<Row>::with_capacity(full_sensors.len());

  for name_datapoint_pair in full_sensors.iter() {
    let name: &String = &name_datapoint_pair.name;
    let datapoint: &SensorDatapoint = &name_datapoint_pair.value;

    // Determine rolling change of the measurement value via value - rolling
    // average of value as calculated by update_information And color code
    // the change based on it's magnitude and sign (increasing / decreasing)
    let d_v = datapoint.measurement.value - datapoint.rolling_average;
    let d_v_style: Style;

    // As values can have vastly differing units, the color code change is 1% of
    // the value, with a minimum change threshold of 0.01 if the value is less
    // than 1
    let value_magnitude_min: f64 = 1.0;
    let value_magnitude: f64 =
      if datapoint.rolling_average.abs() > value_magnitude_min {
        datapoint.rolling_average.abs()
      } else {
        value_magnitude_min
      };

    // If the change is > 1% the rolling averages value, then it's considered
    // significant enough to highlight. Since sensors have a bigger
    // potential range, a flat delta threshold is a bad idea as it would require
    // configuration.
    if d_v.abs() / value_magnitude < 0.01 {
      d_v_style = data_style;
    } else if d_v > 0.0 {
      d_v_style = normal_style.fg(Color::Green);
    } else {
      d_v_style = normal_style.fg(Color::Red);
    }

    rows.push(
      Row::new(vec![
        Cell::from(
          // Sensor Name
          Span::from(name.clone())
            .style(normal_style)
            .bold()
            .into_right_aligned_line(),
        ),
        Cell::from(
          // Measurement value
          Span::from(format!("{:.3}", datapoint.measurement.value))
            .into_right_aligned_line()
            .style(data_style),
        ),
        Cell::from(
          // Measurement unit
          Span::from(format!("{}", datapoint.measurement.unit))
            .into_left_aligned_line()
            .style(data_style.fg(GREY)),
        ),
        Cell::from(
          // Rolling Change of value (see update_information)
          Span::from(format!("{:+.3}", d_v)).into_left_aligned_line(),
        )
        .style(d_v_style),
      ])
      .style(normal_style),
    );
  }

  //  ~Fixed Lengths with some room to expand
  let widths = [
    Constraint::Min(12),
    Constraint::Min(10),
    Constraint::Length(5),
    Constraint::Min(14),
  ];

  //  Make the table itself
  let sensor_table: Table<'_> = Table::new(rows, widths)
    .style(normal_style)
    // It has an optional header, which is simply a Row always visible at the
    // top.
    .header(
      Row::new(vec![
        Span::from("Name").into_right_aligned_line(),
        Span::from("Value").into_right_aligned_line(),
        Span::from("Unit").into_centered_line(),
        Span::from("Rolling Change").into_centered_line(),
      ])
      .style(Style::new().bold())
      // To add space between the header and the rest of the rows, specify the
      // margin
      .bottom_margin(1),
    )
    // As any other widget, a Table can be wrapped in a Block.
    .block(Block::default().title("Sensors").borders(Borders::ALL))
    // The selected row and its content can also be styled.
    .highlight_style(Style::new().reversed())
    // ...and potentially show a symbol in front of the selection.
    .highlight_symbol(">>");

  //  Render
  f.render_widget(sensor_table, area);
}
