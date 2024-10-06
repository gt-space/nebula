use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::{self, Shared};
use axum::{extract::State, Json};
use common::comm::Log;

/// Simple function for forwarding a log request to the log controller
pub async fn post_log_generic(
  State(shared): State<Shared>,
  Json(request): Json<Log>,
) -> server::Result<()> {
  let mut log: Log = request.clone();
  if log.time_stamp == UNIX_EPOCH {
    // If receiving log with a timestamp of 0, use the current time as the
    // timestamp
    log.time_stamp = SystemTime::now();
  }
  shared.logs.0.lock().await.log(log);
  Ok(())
}
