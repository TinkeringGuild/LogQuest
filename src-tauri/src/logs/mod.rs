pub mod active_character_detection;
pub mod log_event_broadcaster;
pub mod log_file_cursor;
pub mod log_line_stream;

use fancy_regex::Regex;

/// This determines how many Lines and LogFileEvents can be buffered
const FILESYSTEM_EVENT_QUEUE_SIZE: usize = 500;

#[derive(thiserror::Error, Debug, Clone)]
#[error("Could not parse log file line: `{0}`")]
struct LogLineParseError(String);

lazy_static::lazy_static! {
  pub static ref LOG_FILENAME_PATTERN: Regex =
    Regex::new(r"(?:\A|[\\/])eqlog_([^_]+)_([^.]+)\.txt$").unwrap();
}

#[derive(Debug, Clone)]
pub enum LogFileEvent {
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
  fn from(raw_line: &str) -> Result<Self, LogLineParseError> {
    if !raw_line.starts_with("[") {
      return Err(LogLineParseError(raw_line.to_owned()));
    }
    let Some((datetime_end, _)) = raw_line.char_indices().find(|(_i, c)| *c == ']') else {
      return Err(LogLineParseError(raw_line.to_owned()));
    };
    let raw_datetime = raw_line[1..datetime_end].to_owned();
    let content = &raw_line[datetime_end + 2..];
    let content = content.replace("&PCT;", "%");
    Ok(Line {
      content,
      raw_datetime,
    })
  }
}
