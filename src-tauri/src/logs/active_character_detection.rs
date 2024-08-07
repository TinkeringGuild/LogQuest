use super::{LogFileEvent, LOG_FILE_PATTERN};
use futures::FutureExt as _;
use std::ops::Deref as _;
use std::path::PathBuf;
use tauri::async_runtime::spawn;
use tokio::select;
use tokio::sync::{broadcast, oneshot, watch};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct Character {
  pub name: String,
  #[allow(unused)]
  pub server: String,
  pub log_file_path: String,
}
impl Character {
  pub fn log_file_pathbuf(&self) -> PathBuf {
    PathBuf::from(&self.log_file_path)
  }
}

pub struct ActiveCharacterDetector {
  watcher: watch::Receiver<anyhow::Result<Option<Character>>>,
  stopper: oneshot::Sender<()>,
}

impl ActiveCharacterDetector {
  pub fn start(subscription: broadcast::Receiver<LogFileEvent>) -> Self {
    let (change_sender, change_receiver) =
      watch::channel::<anyhow::Result<Option<Character>>>(Ok(None));
    let (stop_sender, stop_receiver) = oneshot::channel::<()>();

    debug!("Spawning tokio task");
    spawn(determine_active_character_from_file_events_async(
      subscription,
      change_sender,
      stop_receiver,
    ));

    Self {
      watcher: change_receiver,
      stopper: stop_sender,
    }
  }

  pub fn current(&self) -> Option<Character> {
    match self.watcher.borrow().deref() {
      Ok(v) => v.clone(),
      Err(_) => None,
    }
  }

  pub fn changed(
    &mut self,
  ) -> impl std::future::Future<Output = Result<(), watch::error::RecvError>> + '_ {
    self.watcher.changed()
  }

  pub fn stop(self) {
    debug!("Sending stop signal");
    if let Err(e) = self.stopper.send(()) {
      warn!("Got an error sending the stop signal: {e:#?}");
    }
  }
}

impl Character {
  /// This method expects the input to be a pre-validated path, otherwise it panics.
  fn from(input: &str) -> Self {
    let captures = LOG_FILE_PATTERN
      .captures(input)
      .expect("Character struct given invalid file path!")
      .unwrap();
    Self {
      name: captures
        .get(1)
        .expect("NO CHAR NAME CAPTURE")
        .as_str()
        .to_owned(),
      server: captures
        .get(2)
        .expect("NO SERVER NAME CAPTURE")
        .as_str()
        .to_owned(),
      log_file_path: input.to_owned(),
    }
  }
}

async fn determine_active_character_from_file_events_async(
  mut rx_fs_events: broadcast::Receiver<LogFileEvent>,
  change_sender: tokio::sync::watch::Sender<anyhow::Result<Option<Character>>>,
  stop_receiver: tokio::sync::oneshot::Receiver<()>,
) {
  debug!("Started async active character detector task");
  let mut current_active_file: Option<String> = None;

  // the fuse makes the oneshot receiver usable in a loop (avoiding a move error with previous loop iterations)
  let mut stop_receiver = stop_receiver.fuse();

  debug!("Starting select loop for LogFileEvents");
  loop {
    // TODO: This should maybe debounce a timeout if a file hasn't been updated after N seconds.
    select! {
        _ = &mut stop_receiver => {
            debug!("Received stop signal for LogFileEvent loop");
            break;
        },

        log_file_event = rx_fs_events.recv() => {
          debug!("Got a new LogFileEvent: {log_file_event:?}");
          match log_file_event {
            // if None is received, the channel is closed
            Err(broadcast::error::RecvError::Closed) => {
              debug!("LogFileEvent channel closed");
              break;
            },

            Err(broadcast::error::RecvError::Lagged(num_behind)) => {
              warn!("Active Character Detector lagged behind filesystem events by {num_behind} messages");
            }
            Ok(LogFileEvent::Err(e)) => {
              error!("Received an error while detecting the active character using filesystem events: {e}");
              break;
            },

            Ok(LogFileEvent::Created(event_path) | LogFileEvent::Updated(event_path)) => {
              debug!("Active Character Detector encountered an event for {}", &event_path);
              if let Some(current) = current_active_file.as_deref() {
                if current == &event_path {
                  continue;
                }
              }
              let cnws = Character::from(&event_path);
              info!("Sending active character change: {cnws:#?}");
              if let Err(e) = change_sender.send(Ok(Some(cnws))) {
                warn!("Couldn't send a change: {e:#?}");
                break;
              }
              current_active_file = Some(event_path);
            },

            Ok(LogFileEvent::Deleted(deleted_path)) => {
              debug!("Active Character Detector encountered a Deleted event for {}", &deleted_path);
              if let Some(current) = current_active_file.as_deref() {
                if current == deleted_path {
                  current_active_file = None;
                  if let Err(e) = change_sender.send(Ok(None)) {
                    warn!("Couldn't send a change: {e:#?}");
                    break;
                  }
                }
              }
            }
          }
        }
    }
  }
  debug!("Character change detection loop complete");
}
