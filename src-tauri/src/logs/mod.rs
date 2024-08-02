pub mod active_character_detection;
pub mod log_event_broadcaster;
pub mod log_reader;

use anyhow::bail;
use regex::Regex;

/// This determines how many Lines and LogFileEvents can be buffered
const FILESYSTEM_EVENT_QUEUE_SIZE: usize = 500;

lazy_static::lazy_static! {
  pub static ref LOG_FILE_PATTERN: Regex =
    Regex::new(r"(?:\A|[\\/])eqlog_([^_]+)_([^.]+)\.txt$").unwrap();
}

#[derive(Debug, Clone)]
pub enum LogFileEvent {
  Err(String),
  Created(String),
  Updated(String),
  Deleted(String),
}

/// An EverQuest log line looks like the following:
/// [Thu Jul 18 17:35:14 2024] You gain experience!!
/// This Line struct separates out the content from the datetime component.
#[derive(Debug, Clone)]
pub struct Line {
  pub content: String,
  #[allow(unused)]
  pub raw_datetime: String,
}

impl Line {
  // This method does not use regular expressions to separate the datetime from the content because it
  // is in the critical path of the application and the logic is dead-simple.
  fn from(raw_line: &str) -> anyhow::Result<Self> {
    if !raw_line.starts_with("[") {
      bail!("Encountered invalid EverQuest log line");
    }
    let Some((datetime_end, _)) = raw_line.char_indices().find(|(_i, c)| *c == ']') else {
      bail!("Encountered invalid EverQuest log line");
    };
    let raw_datetime = raw_line[1..datetime_end].to_owned();
    let content = raw_line[datetime_end + 2..].to_owned();
    Ok(Line {
      content,
      raw_datetime,
    })
  }
}
