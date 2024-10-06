use super::*;
use ratatui::{prelude::*, widgets::*};
use std::{time::SystemTime, vec::Vec};

pub fn logs_tab(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  let horizontal = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Fill(1),
      Constraint::Length(200),
      Constraint::Fill(1),
    ]) // Fill to the identical width of the other stuff
    .split(area);

  draw_empty(f, horizontal[0]);
  draw_empty(f, horizontal[2]);

  let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Fill(4), Constraint::Fill(2)])
    .split(horizontal[1]);

  draw_log_section(f, vertical[0], tui_data);

  draw_console_history_section(f, vertical[1], tui_data);
}

// Returns Header Style, First Line Style, and Body Style
fn log_section_colors(
  log_type: &LogType,
  log_num: usize,
) -> (Style, Style, Style) {
  let std_alt_bg: [Color; 2] =
    [Color::from_u32(0x00282828), Color::from_u32(0x000f0f0f)];

  let err_alt_bg: [Color; 2] =
    [Color::from_u32(0x005e2a2a), Color::from_u32(0x00351919)];

  let scs_alt_bg: [Color; 2] =
    [Color::from_u32(0x002a5e2a), Color::from_u32(0x00193519)];

  let alt_fg: [Color; 2] =
    [Color::from_u32(0x00ffffff), Color::from_u32(0x00bdbdbd)];

  let alt: usize = if (log_num & 1) != 0 { 1 } else { 0 };
  match log_type {
    LogType::Debug => (
      YJSP_STYLE.bg(std_alt_bg[alt]),
      YJSP_STYLE.bg(std_alt_bg[alt]).fg(DESATURATED_YJSP_YELLOW),
      YJSP_STYLE.bg(std_alt_bg[alt]).fg(alt_fg[alt]),
    ),
    LogType::Standard => (
      YJSP_STYLE.bg(std_alt_bg[alt]),
      YJSP_STYLE.bg(std_alt_bg[alt]).fg(DESATURATED_YJSP_YELLOW),
      YJSP_STYLE.bg(std_alt_bg[alt]).fg(alt_fg[alt]),
    ),
    LogType::Error => (
      YJSP_STYLE.bg(err_alt_bg[alt]),
      YJSP_STYLE.bg(err_alt_bg[alt]).fg(DESATURATED_YJSP_YELLOW),
      YJSP_STYLE.bg(err_alt_bg[alt]).fg(alt_fg[alt]),
    ),
    LogType::Success => (
      YJSP_STYLE.bg(scs_alt_bg[alt]),
      YJSP_STYLE.bg(scs_alt_bg[alt]).fg(DESATURATED_YJSP_YELLOW),
      YJSP_STYLE.bg(scs_alt_bg[alt]).fg(alt_fg[alt]),
    ),
  }
}

fn draw_log_section(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  let filtered = &tui_data.filtered_logs; // Make rows

  let mut rows: Vec<Row> = Vec::<Row>::with_capacity(filtered.len() * 4); // rough estimate of size

  let max_number: usize =
    if area.height > 4 { area.height - 4 } else { 1 } as usize;

  let mut total_line_count: usize = 0;

  let mut log_count: usize = tui_data.last_log_count - filtered.len();

  for log_item in filtered {
    // While this isn't the most efficient method of doing this, it's much
    // more modular and clean than a single pass.
    // If it becomes an issue, we'll change it.
    let mut log_lines: Vec<&str>;
    let header_line_count: usize;
    if log_item.header.is_empty() {
      log_lines = log_item.contents.split("\n").collect::<Vec<_>>();
      header_line_count = 1;
    } else if log_item.contents.is_empty() {
      log_lines = log_item.header.split("\n").collect::<Vec<_>>();
      header_line_count = log_lines.len();
    } else {
      log_lines = log_item.header.split("\n").collect::<Vec<_>>();
      header_line_count = log_lines.len();
      log_lines.extend(log_item.contents.split("\n"));
    }

    let mut first: bool = true; // using a bool here feels dumb

    let (meta_style, header_style, body_style) =
      log_section_colors(&log_item.log_type, log_count);

    for (line_index, line) in log_lines.into_iter().enumerate() {
      let style = if line_index < header_line_count {
        header_style
      } else {
        body_style
      };

      if first {
        first = false;
        rows.push(
          Row::new(vec![
            Cell::from(
              Span::from(log_item.source.clone()).into_left_aligned_line(),
            ), // Source
            Cell::from(
              Span::from(log_item.log_type.to_string()).into_centered_line(),
            ), // Type
            Cell::from(
              Span::from(
                log_item
                  .time_stamp
                  .duration_since(SystemTime::UNIX_EPOCH)
                  .expect("How did you get a time before UNIX EPOCH????")
                  .as_millis()
                  .to_string(),
              )
              .into_right_aligned_line(),
            ), // Time (TODO : make more readable)
            Cell::from(
              Span::from(log_item.log_category.to_string())
                .into_centered_line(),
            ), // Category
            Cell::from(Span::from(line).style(style)),
          ])
          .style(meta_style),
        );
      } else {
        rows.push(
          Row::new(vec![
            Cell::from(""), // Source not used
            Cell::from(""), // Type not used
            Cell::from(""), // Time not used
            Cell::from(""), // Category not used
            Cell::from(line),
          ])
          .style(style),
        );
      }
      total_line_count += 1;
    }

    log_count += 1;
  }

  let widths: [Constraint; 5] = [
    Constraint::Length(10),
    Constraint::Length(10),
    Constraint::Length(12),
    Constraint::Length(10),
    Constraint::Fill(1),
  ];

  // because the scroll functions are not behaving how they should, I'll just
  // manually slice this.
  let first_index: usize = if total_line_count > max_number {
    total_line_count - max_number
  } else {
    0
  };

  let event_log: Table<'_> = Table::new(rows.split_off(first_index), widths)
    .style(YJSP_STYLE)
    .fg(YJSP_YELLOW)
    // It has an optional header, which is simply a Row always visible at the
    // top.
    .header(
      Row::new(vec![
        Span::from("Source").into_left_aligned_line(),
        Span::from("Type").into_centered_line(),
        Span::from("Time").into_centered_line(),
        Span::from("Category").into_left_aligned_line(),
        Span::from("Event").into_left_aligned_line(),
      ])
      .style(Style::new().bold())
      // To add space between the header and the rest of the rows, specify the
      // margin
      .bottom_margin(1),
    )
    // As any other widget, a Table can be wrapped in a Block.
    .block(Block::default().title("Event Log").borders(Borders::ALL))
    // The selected row and its content can also be styled.
    .highlight_style(Style::new().reversed())
    // ...and potentially show a symbol in front of the selection.
    .highlight_symbol(">>");

  f.render_widget(event_log, area);
}

fn draw_console_history_section(f: &mut Frame, area: Rect, tui_data: &TuiData) {
  //  Get valve states from TUI
  let command_logs = &tui_data.command_log;
  let command_queue = &tui_data.sequence_queue;

  // Make rows
  let mut rows: Vec<Row> =
    Vec::<Row>::with_capacity(command_logs.len() + command_queue.len());

  let max_number: usize =
    if area.height > 4 { area.height - 4 } else { 1 } as usize;

  let mut count: usize = 0;

  //  Get base style used in this row based on the actual (derived) state of the
  // valve
  let normal_style = YJSP_STYLE.fg(WHITE);
  let name_style = YJSP_STYLE.fg(WHITE);

  // Generate results
  let sending_style = YJSP_STYLE.fg(GREY);
  let loading_anim: [char; 4] = ['\\', '|', '/', '-'];
  let sending_line: Line = Span::from(
    loading_anim[(tui_data.frame >> 2) % loading_anim.len()].to_string()
      + "Sending",
  )
  .into_left_aligned_line()
  .style(sending_style);

  let sent_style = YJSP_STYLE.fg(DESATURATED_GREEN);
  let sent_line: Line = Span::from("Sent")
    .into_left_aligned_line()
    .style(sent_style);

  let failure_style = YJSP_STYLE.fg(DESATURATED_RED);
  let _failed_line: Line = Span::from("Failed")
    .into_left_aligned_line()
    .style(failure_style);
  let failed_send_line: Line = Span::from("FailSend")
    .into_left_aligned_line()
    .style(failure_style);

  for command in command_queue {
    // Make the actual row of info
    rows.push(
      Row::new(vec![
        Cell::from(
          Span::from(command.script.clone())
            .into_right_aligned_line()
            .style(name_style),
        ), // Name of Valve
        //Cell::from(Span::from(timeout.as_millis().to_string()).
        // into_right_aligned_line().style(name_style))
        Cell::from(sending_line.clone()),
      ])
      .style(normal_style),
    );

    count += 1;

    if count >= max_number {
      break;
    }
  }

  for command in command_logs {
    if count >= max_number {
      break;
    }

    // Make the actual row of info
    rows.push(
      Row::new(vec![
        Cell::from(
          Span::from(command.script.clone())
            .into_right_aligned_line()
            .style(name_style),
        ), // Name of Valve
        //Cell::from(Span::from(timeout.as_millis().to_string()).
        // into_right_aligned_line().style(name_style))
        Cell::from(match command.result {
          SequenceSendResults::NoResult => sending_line.clone(),
          SequenceSendResults::Sent => sent_line.clone(),
          SequenceSendResults::FailedSend => failed_send_line.clone(),
          _ => Span::from("Unknown")
            .into_left_aligned_line()
            .style(sending_style),
        }),
      ])
      .style(normal_style),
    );

    count += 1;
  }

  rows.reverse(); // TODO : BE ACTUALLY OPTIMIZED

  let widths = [Constraint::Fill(1), Constraint::Length(8)];

  let history_table: Table<'_> = Table::new(rows, widths)
    .style(YJSP_STYLE)
    .fg(if tui_data.is_connected {
      YJSP_YELLOW
    } else {
      DESATURATED_RED
    })
    // It has an optional header, which is simply a Row always visible at the
    // top.
    .header(
      Row::new(vec![
        Span::from("Command").into_right_aligned_line(),
        Span::from("Status").into_centered_line(),
      ])
      .style(Style::new().bold())
      // To add space between the header and the rest of the rows, specify the
      // margin
      .bottom_margin(1),
    )
    // As any other widget, a Table can be wrapped in a Block.
    .block(
      Block::default()
        .title("Remote Console Commands")
        .borders(Borders::ALL),
    )
    // The selected row and its content can also be styled.
    .highlight_style(Style::new().reversed())
    // ...and potentially show a symbol in front of the selection.
    .highlight_symbol(">>");

  f.render_widget(history_table, area);
}
