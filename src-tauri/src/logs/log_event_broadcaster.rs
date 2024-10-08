use super::{LogFileEvent, FILESYSTEM_EVENT_QUEUE_SIZE, LOG_FILENAME_PATTERN};
use notify::{RecommendedWatcher, Watcher};
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};
use tokio::sync::broadcast;
use tracing::{debug, error};

pub struct LogEventBroadcaster {
  logs_dir: PathBuf,
  watcher: RecommendedWatcher,
  tx: broadcast::Sender<Result<LogFileEvent, NotifyError>>,
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Notify error")]
pub struct NotifyError(Arc<notify::Error>); // notify::Error does not implement Clone
impl From<notify::Error> for NotifyError {
  fn from(err: notify::Error) -> Self {
    Self(Arc::new(err))
  }
}

impl LogEventBroadcaster {
  pub fn new(logs_dir: &Path) -> Result<Self, NotifyError> {
    let (tx, _rx) =
      broadcast::channel::<Result<LogFileEvent, NotifyError>>(FILESYSTEM_EVENT_QUEUE_SIZE);
    let callback = new_notify_event_handler(tx.clone());
    // TODO! Should use a notify::Config
    let watcher = notify::recommended_watcher(callback)?;
    debug!(
      "Created (unstarted) LogEventBroadcaster for dir: {}",
      logs_dir.display()
    );
    Ok(Self {
      logs_dir: logs_dir.to_owned(),
      watcher,
      tx,
    })
  }

  pub fn start(&mut self) -> Result<(), notify::Error> {
    debug!("Starting LogEventBroadcaster");
    self
      .watcher
      .watch(&self.logs_dir, notify::RecursiveMode::NonRecursive)
  }

  pub fn subscribe(&self) -> broadcast::Receiver<Result<LogFileEvent, NotifyError>> {
    self.tx.subscribe()
  }

  pub fn stop(mut self) -> Result<(), notify::Error> {
    self.watcher.unwatch(&self.logs_dir)
  }

  pub fn sender(&self) -> broadcast::Sender<Result<LogFileEvent, NotifyError>> {
    self.tx.clone()
  }
}

fn new_notify_event_handler(
  sender: broadcast::Sender<Result<LogFileEvent, NotifyError>>,
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
              _ = sender.send(Ok(e));
            });
        }
        EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any) => {
          // debug!("Filesystem Modify event");
          event
            .paths
            .iter()
            .filter(|p| is_valid_log_file_name(p))
            .map(|p| LogFileEvent::Updated(path_string(p)))
            .for_each(|e| {
              _ = sender.send(Ok(e));
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
              _ = sender.send(Ok(e));
            });
        }
        _ => {}
      },
      Err(error) => {
        error!("Notify error! {error:#?}");
        _ = sender.send(Err(error.into()));
      }
    }
  }
}

fn is_valid_log_file_name(path: &Path) -> bool {
  let path = path.to_string_lossy();
  LOG_FILENAME_PATTERN.is_match(&path).is_ok_and(|b| b)
}

fn path_string(path: &Path) -> String {
  path
    .canonicalize()
    .unwrap_or_else(|_| path.to_owned())
    .to_string_lossy()
    .into_owned()
}
