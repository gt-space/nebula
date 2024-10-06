use common::comm::{LogCategory, LogType};
use serde_json::json;
use std::time::{Duration, SystemTime};

/// Function for pushing a log to servo
pub fn log(
  log_type: Option<LogType>,
  log_category: Option<LogCategory>,
  source: Option<String>,
  header: Option<String>,
  contents: Option<String>,
) -> anyhow::Result<()> {
  let client = reqwest::blocking::Client::new();
  let _ = client
    .post("http://localhost:7200/logging/log")
    .json(&json!({
      "log_type" : log_type.unwrap_or(LogType::Standard),
      "log_category" : log_category.unwrap_or(LogCategory::Unknown),
      "time_stamp" : SystemTime::now(),
      "source" : source.unwrap_or(String::from("servo")),
      "header" : header.unwrap_or(if contents.is_some() { String::from("Empty Log") } else  { String::from("") } ),
      "contents" : contents.unwrap_or_default()
    }))
    .timeout(Duration::from_secs(2))
    .send()?;

  Ok(())
}
