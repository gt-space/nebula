mod display;

mod tabs;

mod tui_data;

pub use display::display;

use tui_data::*;

/// A set of imports, constants, and generitcally used functions that are
/// utilized throughout all parts of the TUI (excludes colors, which are in
/// `colors_and_styles`)
mod generic {
  use super::colors_and_styles::*;
  use crate::server::Shared;
  use std::sync::Arc;

  pub use common::comm::{Log, LogCategory, LogType, ValveState};
  use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Row, Table},
    Frame,
  };

  /// Submits a log to the log controller asynchronously with the header
  /// automatically containing the phrase : "TUI Log : " at it's start
  pub async fn _log_tui(
    log_type: LogType,
    header: String,
    body: String,
    shared: Arc<Shared>,
  ) {
    let mut logs = shared.logs.0.lock().await;
    logs.log_here(
      log_type,
      LogCategory::Unknown,
      format!("TUI Log : {}", header),
      body,
    );
  }

  #[derive(Copy, Clone)]
  /// The "modes" of the TUI (pages, subpages, etc)
  ///
  /// Used to communicate what the TUI should render
  pub enum Modes {
    /// The main menu of the TUI, displays general stats and connections
    Home,
    /// The logging section of the TUI, displays
    Logs,
  }

  #[derive(Copy, Clone, PartialEq)]
  /// The state of the console for the TUI
  ///
  /// Currently an enum to more easily allow the ability to swap what the
  /// console is targeting in the future (a la there is a flight *and* ground
  /// computer) if so desired.
  pub enum TUIConsoleState {
    /// No visible or interactable console
    Hidden,
    /// Console that sends commands to the flight comptuer is visible and
    /// interactable
    Flight,
  }

  /// Draws an empty table within an area. Used to fill a region with the
  /// `YJSP_STYLE`'s background
  pub fn draw_empty(f: &mut Frame, area: Rect) {
    let widths = [Constraint::Fill(1)];

    let empty_table: Table<'_> = Table::new(Vec::<Row>::new(), widths)
      .style(YJSP_STYLE)
      .header(
        Row::new(vec![Span::from("").into_centered_line()])
          .style(Style::new().bold()),
      );

    f.render_widget(empty_table, area);
  }
}

use generic::*;

#[allow(dead_code)]
/// module for all default colors and styles
///
/// In seperate module to have #[allow(dead_code)] not affect generic
mod colors_and_styles {
  use ratatui::style::{Color, Style};
  /// The yellow used for a majority of displayed objects
  ///
  /// Typically only used for very important things such as TUI element borders,
  /// device names, table headers, etc
  pub const YJSP_YELLOW: Color = Color::from_u32(0x00ffe659);

  /// A desaturated version of YJSP_YELLOW
  ///
  /// Typically used for elements that are important, but would cause visual
  /// clutter if kept as YJSP_YELLOW (Such as header lines in the logs tab)
  pub const DESATURATED_YJSP_YELLOW: Color = Color::from_u32(0x00fcee9f);

  /// A decently vibrant Red
  ///
  /// Poor on the eyes for text, use `DESATURATED_RED` instead if ease of
  /// readbility is important / for places where color cannot communicate
  /// all information : a la logs with custom headers.
  pub const RED: Color = Color::from_u32(0x00db2c2c);

  /// A pretty bright white
  ///
  /// Not #FFFFFF white however, only #EEEEEE (or {238, 238, 238} in RGB)
  pub const WHITE: Color = Color::from_u32(0x00eeeeee);

  /// Pure black
  ///
  /// Literally just #000000
  pub const BLACK: Color = Color::from_u32(0);

  /// A bright purple used when an unexpected "unknown" value occurs
  /// in display logic
  ///
  /// Typically used as a fallthrough for match functions they don't
  /// throw errors if one is added values to the enum being matched
  pub const UNKNOWN_PURPLE: Color = Color::from_u32(0x00da3ee6);

  /// A middle of the road grey
  ///
  /// #BBBBBB
  pub const GREY: Color = Color::from_u32(0x00bbbbbb);

  /// A dark grey
  ///
  /// #444444
  pub const DARK_GREY: Color = Color::from_u32(0x00444444);

  /// A desaturated green designed to be easy on the eyes when displayed
  ///
  /// Actually good for text where the ease of readbility of text is important
  pub const DESATURATED_GREEN: Color = Color::from_u32(0x007aff85);

  /// A desaturated red designed to be easy on the eyes when displayed
  ///
  /// Actually good for text where the ease of readbility of text is important
  pub const DESATURATED_RED: Color = Color::from_u32(0x00ff5959);

  /// A desaturated blue designed to be easy on the eyes when displayed
  ///
  /// Actually good for text where the ease of readbility of text is important
  pub const DESATURATED_BLUE: Color = Color::from_u32(0x0075a8ff);

  /// The default style used for the entire TUI. A vid
  pub const YJSP_STYLE: Style = Style::new().bg(BLACK).fg(YJSP_YELLOW);
}

pub use colors_and_styles::*;
