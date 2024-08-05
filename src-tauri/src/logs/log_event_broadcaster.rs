use super::{LogFileEvent, FILESYSTEM_EVENT_QUEUE_SIZE, LOG_FILE_PATTERN};
use crate::common::path_string;
use notify::{RecommendedWatcher, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use tracing::{debug, error};

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

  pub fn stop(mut self) -> Result<(), notify::Error> {
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
            .filter(|p| is_valid_log_file_name(p))
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
            .filter(|p| is_valid_log_file_name(p))
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
            .filter(|p| is_valid_log_file_name(p))
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

fn is_valid_log_file_name(path: &Path) -> bool {
  let path = path.to_string_lossy();
  LOG_FILE_PATTERN.is_match(&path).is_ok_and(|b| b)
}
