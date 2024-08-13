use super::log_event_broadcaster::NotifyError;
use super::{Line, LogFileEvent, FILESYSTEM_EVENT_QUEUE_SIZE};
use futures::FutureExt as _;
use std::ffi::OsStr;
use std::path::Path;
use tauri::async_runtime::spawn;
use tokio::io::{AsyncBufReadExt as _, AsyncSeekExt as _};
use tokio::select;
use tokio::sync::{broadcast, oneshot};
use tracing::{debug, error, warn};

pub struct LogReader {
  // pub path: PathBuf,
  tx: broadcast::Sender<Line>,
  stopper: Option<oneshot::Sender<()>>,
}

impl LogReader {
  pub fn start(
    log_file_path: &Path,
    mut event_reader: broadcast::Receiver<Result<LogFileEvent, NotifyError>>,
  ) -> Self {
    tracing::info!("In LogReader start");
    let (tx, _rx) = broadcast::channel::<Line>(FILESYSTEM_EVENT_QUEUE_SIZE);
    let (tx_stop, rx_stop) = oneshot::channel::<()>();
    let mut rx_stop = rx_stop.fuse();
    let tx_later = tx.clone();
    let log_file_path = log_file_path.to_owned();

    debug!("Spawning LogReader task");
    spawn(async move {
      let mut file = match tokio::fs::File::open(&log_file_path).await {
        Ok(f) => f,
        Err(e) => {
          error!(
            "Could not open log file at {}. Error: {e:#?}",
            log_file_path.display()
          );
          return;
        }
      };
      if let Err(e) = file.seek(tokio::io::SeekFrom::End(0)).await {
        error!(
          "Could not seek to end of file {}: {e:#?}",
          log_file_path.display()
        );
      }
      let reader = tokio::io::BufReader::new(file);
      let mut lines = reader.lines();

      debug!("Start LogReader loop");
      'select: loop {
        select! {
          _ = &mut rx_stop => {
            debug!("No longer watching log file: {}", log_file_path.display());
            break;
          }
          next = event_reader.recv() => {
            match next {
              Err(broadcast::error::RecvError::Closed) => break, // TODO log something?
              Err(broadcast::error::RecvError::Lagged(num_missed)) => {
                warn!(
                  "Log reader select loop for {} lagged behind by {} messages",
                  log_file_path.display(),
                  num_missed
                );
              },
              Ok(Ok(LogFileEvent::Updated(event_path))) => {
                debug!("Detected UPDATE: {event_path}");
                if OsStr::new(&event_path) != log_file_path { continue; }
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
              Ok(Ok(LogFileEvent::Created(event_path))) => {
                debug!("Detected CREATE: {event_path}");
                if OsStr::new(&event_path) != log_file_path { continue; }
                // TODO: does a "created" event occur when a file is overwritten? If so,
                // could this cause an issue with an open file descriptor?
              },
              Ok(Ok(LogFileEvent::Deleted(event_path))) => {
                debug!("Detected DELETE: {event_path}");
                if OsStr::new(&event_path) != log_file_path { continue; }
                warn!("Log file {} was deleted while it was being watched", log_file_path.display());
                break;
              },
              Ok(Err(notify_error)) => {
                // Uncertain whether a NotifyError should be considered recoverable.
                error!("Encountered a Notify error! {notify_error:?}");
              }
            }
          }
        }
      }
      debug!(
        "Log reader select loop finished for file {}",
        log_file_path.display()
      );
    });

    Self {
      // path: logfile_pathbuf,
      tx: tx_later,
      stopper: Some(tx_stop),
    }
  }

  pub fn idle() -> Self {
    let (tx, _rx) = broadcast::channel::<Line>(1);
    Self { tx, stopper: None }
  }

  pub fn subscribe(&self) -> broadcast::Receiver<Line> {
    self.tx.subscribe()
  }

  pub fn stop(self) {
    if let Some(stopper) = self.stopper {
      let _ = stopper.send(());
    }
  }
}
