use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use std::path::Path;

const RANDOM_ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

lazy_static! {
  pub static ref LOG_FILE_PATTERN: Regex =
    Regex::new(r"(?:^|\b)eqlog_([^_]+)_([^.]+)\.txt$").unwrap();
}

pub fn path_string(path: &Path) -> String {
  let path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
  path.to_string_lossy().to_string()
}

pub fn random_id(length: u16) -> String {
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let random_index = rng.gen_range(0..RANDOM_ID_CHARSET.len());
      RANDOM_ID_CHARSET[random_index] as char
    })
    .collect()
}

// pub fn unix_time_now() -> f64 {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_secs_f64()
// }
