pub mod duration;
pub mod serializable_regex;
pub mod timestamp;

use rand::Rng;
use std::path::Path;

pub(crate) fn path_string(path: &Path) -> String {
  let path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
  path.to_string_lossy().to_string()
}

pub(crate) fn random_id(length: u16) -> String {
  const RANDOM_ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let random_index = rng.gen_range(0..RANDOM_ID_CHARSET.len());
      RANDOM_ID_CHARSET[random_index] as char
    })
    .collect()
}

pub(crate) fn fatal_error(message: &str) {
  tracing::error!(message);
  std::process::exit(2);
}
