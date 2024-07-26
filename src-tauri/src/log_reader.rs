use crate::common::{path_string, LOG_FILE_PATTERN};
use anyhow::bail;
use futures::FutureExt as _;
use notify::{RecommendedWatcher, Watcher};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::runtime::Handle;
use tokio::select;
use tokio::sync::{broadcast, oneshot};
use tokio::{io::AsyncBufReadExt as _, io::AsyncSeekExt as _};
use tracing::{debug, error, warn};

const FILESYSTEM_EVENT_QUEUE_SIZE: usize = 100;

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
  content: String,
  raw_datetime: String,
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

pub struct LogReader {
  // pub path: PathBuf,
  tx: broadcast::Sender<Line>,
  stopper: oneshot::Sender<()>,
}

impl LogReader {
  pub fn start(
    rt: Handle,
    logfile_path: &Path,
    mut event_reader: broadcast::Receiver<LogFileEvent>,
  ) -> Self {
    let (tx, _rx) = broadcast::channel::<Line>(FILESYSTEM_EVENT_QUEUE_SIZE);
    let (tx_stop, rx_stop) = oneshot::channel::<()>();
    let mut rx_stop = rx_stop.fuse();
    let tx_later = tx.clone();

    let logfile_path = logfile_path.to_owned();
    rt.spawn(async move {
      let mut file = match tokio::fs::File::open(&logfile_path).await {
        Ok(f) => f,
        Err(e) => {
          error!(
            "Could not open log file at {}. Error: {e:#?}",
            logfile_path.display()
          );
          return;
        }
      };
      if let Err(e) = file.seek(tokio::io::SeekFrom::End(0)).await {
        error!(
          "Could not seek to end of file {}: {e:#?}",
          logfile_path.display()
        );
      }
      let reader = tokio::io::BufReader::new(file);
      let mut lines = reader.lines();
      'select: loop {
        select! {
          _ = &mut rx_stop => {
            debug!("No longer watching log file: {}", logfile_path.display());
            break;
          }
          next = event_reader.recv() => {
            match next {
              Err(broadcast::error::RecvError::Closed) => break, // TODO log something?
              Err(broadcast::error::RecvError::Lagged(num_missed)) => {
                warn!(
                  "Log reader select loop for {} lagged behind by {} messages",
                  logfile_path.display(),
                  num_missed
                );
              },
              Ok(LogFileEvent::Err(msg)) => {
                error!("Error in log reader select loop: {msg}");
                break;
              },
              Ok(LogFileEvent::Updated(event_path)) => {
                if OsStr::new(&event_path) != logfile_path { continue; }
                while let Ok(Some(next_line)) = lines.next_line().await {
                  let Ok(line) = Line::from(&next_line) else {
                    error!(r#"Encountered an invalid EverQuest log line: "{next_line}""#);
                    continue;
                  };
                  if let Err(e) = tx.send(line) {
                    warn!("Could not deliver new Line to channel: {e:#?}");
                    break 'select;
                  }
                }
              },
              Ok(LogFileEvent::Created(event_path)) => {
                if OsStr::new(&event_path) != logfile_path { continue; }
                // TODO: does a "created" event occur when a file is overwritten? If so,
                // could this cause an issue with an open file descriptor?
              },
              Ok(LogFileEvent::Deleted(event_path)) => {
                if OsStr::new(&event_path) != logfile_path { continue; }
                warn!("Log file {} was deleted while it was being watched", logfile_path.display());
                break;
              },
            }
          }
        }
      }
      debug!(
        "Log reader select loop finished for file {}",
        logfile_path.display()
      );
    });

    Self {
      // path: logfile_pathbuf,
      tx: tx_later,
      stopper: tx_stop,
    }
  }

  pub fn subscribe(&self) -> broadcast::Receiver<Line> {
    self.tx.subscribe()
  }

  pub fn stop(self) {
    let _ = self.stopper.send(());
  }
}

pub struct LogEventBroadcaster {
  logs_dir: PathBuf,
  watcher: RecommendedWatcher,
  tx: broadcast::Sender<LogFileEvent>,
}

impl LogEventBroadcaster {
  pub fn new(logs_dir: &Path) -> anyhow::Result<Self> {
    let (tx, _rx) = broadcast::channel::<LogFileEvent>(FILESYSTEM_EVENT_QUEUE_SIZE);
    let callback = new_notify_event_handler(tx.clone());
    // TODO! Should use a notify::Config
    let watcher = notify::recommended_watcher(callback)?;
    debug!("Watch dir for filesystem events: {}", logs_dir.display());
    Ok(Self {
      logs_dir: logs_dir.to_owned(),
      watcher,
      tx,
    })
  }

  pub fn start(&mut self) -> Result<(), notify::Error> {
    self
      .watcher
      .watch(&self.logs_dir, notify::RecursiveMode::NonRecursive)
  }

  pub fn subscribe(&self) -> broadcast::Receiver<LogFileEvent> {
    self.tx.subscribe()
  }

  fn stop(mut self) -> Result<(), notify::Error> {
    self.watcher.unwatch(&self.logs_dir)
  }
}

fn new_notify_event_handler(
  sender: broadcast::Sender<LogFileEvent>,
) -> impl Fn(Result<notify::Event, notify::Error>) {
  use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind};
  move |res: Result<notify::Event, notify::Error>| {
    match res {
      // This does not handle the case of file-rename events. The only reason someone
      // might do that would be to archive a log by renaming it to something that isn't
      // a LOG_FILE_PATTERN. I can't imagine a scenario when someone would rename a file
      // from one filename that matched LOG_FILE_PATTERN to another file that matched
      // LOG_FILE_PATTERN (especially while LQ is running).
      Ok(event) => match event.kind {
        EventKind::Create(CreateKind::File) => {
          debug!("Filesystem Create event");
          event
            .paths
            .iter()
            .filter(|p| LOG_FILE_PATTERN.is_match(&p.to_string_lossy()))
            .map(|p| LogFileEvent::Created(path_string(p)))
            .for_each(|e| {
              let _ = sender.send(e);
            });
        }
        EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any) => {
          debug!("Filesystem Modify event");
          event
            .paths
            .iter()
            .filter(|p| LOG_FILE_PATTERN.is_match(&p.to_string_lossy()))
            .map(|p| LogFileEvent::Updated(path_string(p)))
            .for_each(|e| {
              let _ = sender.send(e);
            });
        }
        EventKind::Remove(RemoveKind::File) => {
          debug!("Filesystem Remove event");
          event
            .paths
            .iter()
            .filter(|p| LOG_FILE_PATTERN.is_match(&p.to_string_lossy()))
            .map(|p| LogFileEvent::Deleted(path_string(p)))
            .for_each(|e| {
              let _ = sender.send(e);
            });
        }
        _ => {}
      },
      Err(error) => {
        error!("Notify error! {error:#?}");
        let _ = sender.send(error.into());
      }
    }
  }
}

impl<E> From<E> for LogFileEvent
where
  E: std::error::Error,
{
  fn from(e: E) -> Self {
    LogFileEvent::Err(e.to_string())
  }
}
