pub mod duration;
pub mod serializable_regex;
pub mod timestamp;

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UUID(String);

impl UUID {
  pub fn new() -> UUID {
    UUID(::uuid::Uuid::new_v4().to_string())
  }
}

pub(crate) fn path_string(path: &Path) -> String {
  let path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
  path.to_string_lossy().to_string()
}

pub(crate) fn random_id(length: u8) -> String {
  const RANDOM_ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let random_index = rng.gen_range(0..RANDOM_ID_CHARSET.len());
      RANDOM_ID_CHARSET[random_index] as char
    })
    .collect()
}

pub(crate) fn fatal_error<T: ToString>(message: T) -> ! {
  let message: String = message.to_string();
  tracing::error!("{}", message);
  std::process::exit(2);
}

pub(crate) fn ternary<T>(condition: bool, if_true: T, if_false: T) -> T {
  if condition {
    if_true
  } else {
    if_false
  }
}
