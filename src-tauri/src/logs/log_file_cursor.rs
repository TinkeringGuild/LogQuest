use super::LOG_FILENAME_PATTERN;
use std::{collections::HashMap, path::Path};
use tracing::error;

#[derive(Debug, Clone)]
pub struct LogFileCursor {
  pub path: String,
  pub position: u64,
}

pub struct LogFileCursorCache {
  cursors: HashMap<String, LogFileCursorCacheEntry>,
}

pub enum LogFileCursorCacheEntry {
  CachedSize(u64),
  StaleSize,
}

impl LogFileCursorCache {
  pub fn scan_dir<P>(logs_dir: P) -> std::io::Result<Self>
  where
    P: AsRef<Path>,
  {
    let cursors: HashMap<String, LogFileCursorCacheEntry> = logs_dir
      .as_ref()
      .read_dir()?
      .filter_map(|entry| entry.ok())
      .filter_map(|entry| entry.path().canonicalize().ok())
      .map(|path| path.to_string_lossy().into_owned())
      .filter(|path| LOG_FILENAME_PATTERN.is_match(path).unwrap())
      .map(|path| {
        let size = file_size(&path)?;
        Ok((path, LogFileCursorCacheEntry::CachedSize(size)))
      })
      .filter_map(|kvp: std::io::Result<_>| kvp.ok())
      .collect();

    Ok(Self { cursors })
  }

  pub fn get_cursor_and_mark_size_stale<S>(&mut self, path: S) -> std::io::Result<LogFileCursor>
  where
    S: AsRef<str>,
  {
    let path = path.as_ref().to_owned();
    let taken_value = self
      .cursors
      .insert(path.clone(), LogFileCursorCacheEntry::StaleSize);
    match taken_value {
      Some(LogFileCursorCacheEntry::CachedSize(size)) => Ok(LogFileCursor {
        path,
        position: size,
      }),
      None | Some(LogFileCursorCacheEntry::StaleSize) => Ok(LogFileCursor::new(path)?),
    }
  }

  pub fn reset_cursor_position<P>(&mut self, path: P)
  where
    P: AsRef<str>,
  {
    let path: &str = path.as_ref();
    let Ok(size) = file_size(path) else {
      error!("Got an IO error determining file size of {path}. Ignoring file henceforth.");
      self.cursors.remove(path);
      return;
    };
    let _ = self
      .cursors
      .insert(path.to_owned(), LogFileCursorCacheEntry::CachedSize(size));
  }
}

impl LogFileCursor {
  pub fn new<S>(path: S) -> std::io::Result<Self>
  where
    S: AsRef<str>,
  {
    let path: &str = path.as_ref();
    Ok(Self {
      position: file_size(path)?,
      path: path.to_owned(),
    })
  }
}

fn file_size<P>(path: P) -> std::io::Result<u64>
where
  P: AsRef<Path>,
{
  Ok(path.as_ref().metadata()?.len())
}
