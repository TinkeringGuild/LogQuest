pub mod clipboard;
pub mod duration;
pub mod progress_reporter;
pub mod security;
pub mod serializable_regex;
pub mod shutdown;
pub mod timestamp;

use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

lazy_static! {
  #[derive(Serialize)]
  pub static ref LOG_QUEST_VERSION: LogQuestVersion = {
    let crate_version: &str = env!("CARGO_PKG_VERSION");
    let parts: Vec<usize> = crate_version
      .split(".")
      .map(|s| s.parse().unwrap())
      .collect();
    let [major, minor, tiny] = parts.as_slice() else {
      panic!("INVALID VERSION FORMAT DEFINED IN Cargo.toml: {crate_version}");
    };
    LogQuestVersion(*major, *minor, *tiny)
  };

}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct LogQuestVersion(pub usize, pub usize, pub usize); // (major, minor, tiny)

/// Equal to 36
pub const UUID_LEN: usize = "00000000-0000-0000-0000-000000000000".len();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct UUID(String);

impl UUID {
  pub fn new() -> Self {
    Self(::uuid::Uuid::new_v4().to_string())
  }

  pub fn from_str_unchecked(value: &str) -> Self {
    Self(value.to_owned())
  }
}

impl std::fmt::Display for UUID {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl Hash for UUID {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state)
  }
}

impl AsRef<str> for UUID {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

pub fn random_id(length: u8) -> String {
  const RANDOM_ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let random_index = rng.gen_range(0..RANDOM_ID_CHARSET.len());
      RANDOM_ID_CHARSET[random_index] as char
    })
    .collect()
}

pub fn fatal_error<T: ToString>(message: T) -> ! {
  let message: String = message.to_string();
  tracing::error!("{}", message);
  std::process::exit(2);
}

pub fn fatal_if_err<O, E>(result: Result<O, E>) -> O
where
  E: std::error::Error,
{
  match result {
    Ok(ok) => ok,
    Err(e) => fatal_error(e.to_string()),
  }
}

pub fn ternary<T>(condition: bool, if_true: T, if_false: T) -> T {
  if condition {
    if_true
  } else {
    if_false
  }
}

pub fn maybe_blank<S>(option: &Option<S>) -> String
where
  S: ToOwned<Owned = String>,
{
  if let Some(s) = option {
    s.to_owned()
  } else {
    String::new()
  }
}

#[derive(thiserror::Error, Debug)]
pub enum AbsolutePathResolutionError {
  #[error("Could not determine the system home directory")]
  UnknownHomeDir,
}

pub fn absolute_path_handling_tilde<P>(path: P) -> Result<PathBuf, AbsolutePathResolutionError>
where
  P: AsRef<Path>,
{
  let path = path.as_ref();
  let path_string = path.to_string_lossy();

  let path = if path_string.starts_with("~/") {
    if let Some(home_dir) = dirs::home_dir() {
      home_dir.join(&path_string[2..])
    } else {
      return Err(AbsolutePathResolutionError::UnknownHomeDir);
    }
  } else {
    path.to_owned()
  };

  // Attempt to canonicalize, but if it fails then the directory probably
  // doesn't exist yet.
  path.canonicalize().or_else(|_| Ok(path))
}

/// formats numbers with thousands separators. e.g. 12345 = "12,345" and 12 = "12"
pub fn format_integer(number: usize) -> String {
  format!("{number}")
    .chars()
    .rev()
    .collect::<Vec<char>>()
    .into_iter()
    .enumerate()
    .flat_map(|(index, digit)| {
      if index % 3 == 0 && index != 0 {
        vec![',', digit]
      } else {
        vec![digit]
      }
    })
    .rev()
    .collect()
}

pub fn bytes_to_utf8_with_escaped_special_chars(bytes: &[u8]) -> String {
  String::from_utf8_lossy(bytes)
    .chars()
    .flat_map(|c| c.escape_default())
    .collect()
}

#[cfg(unix)]
pub fn file_path_is_executable(path: &str) -> bool {
  use std::fs::metadata;
  use std::os::unix::fs::{MetadataExt as _, PermissionsExt};

  let Ok(metadata) = metadata(path) else {
    return false;
  };

  let permissions = metadata.permissions();
  let mode = permissions.mode();

  // Check if the file is executable by the owner
  let user_uid = unsafe { libc::geteuid() };
  let file_uid = metadata.uid();
  if user_uid == file_uid && mode & 0o100 != 0 {
    return true;
  }

  // Check if the file is executable by the group
  let user_gid = unsafe { libc::getegid() };
  let file_gid = metadata.gid();
  if user_gid == file_gid && mode & 0o010 != 0 {
    return true;
  }

  // Check if the file is executable by others
  mode & 0o001 != 0
}

#[cfg(windows)]
pub fn file_path_is_executable(path: &str) {
  path.ends_with(".exe")
    || path.ends_with(".bat")
    || path.ends_with(".cmd")
    || path.ends_with(".com")
}

mod tests {

  #[test]
  fn test_format_number() {
    use super::format_integer;
    assert_eq!(format_integer(0), "0");
    assert_eq!(format_integer(123), "123");
    assert_eq!(format_integer(1234), "1,234");
    assert_eq!(format_integer(123456789), "123,456,789");
    assert_eq!(
      format_integer(1222333444555666777),
      "1,222,333,444,555,666,777"
    );
  }
}
