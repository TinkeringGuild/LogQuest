use std::sync::atomic::AtomicUsize;

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub enum ProgressUpdate {
  Started,
  Message { text: String, seq: usize },
  Finished { text: String, seq: usize },
}

pub struct ProgressReporter {
  tx: watch::Sender<ProgressUpdate>,
  sequence_number: AtomicUsize,
}

impl ProgressReporter {
  pub fn new() -> (Self, watch::Receiver<ProgressUpdate>) {
    let (tx, rx) = watch::channel(ProgressUpdate::Started);
    let sequence_number = AtomicUsize::new(0);
    (
      Self {
        tx,
        sequence_number,
      },
      rx,
    )
  }

  pub fn update<S>(&self, message: S)
  where
    S: AsRef<str>,
  {
    let update = ProgressUpdate::Message {
      text: message.as_ref().to_owned(),
      seq: self.increment_seq_num(),
    };
    if let Err(_send_error) = self.tx.send(update) {
      error!("Watch SendError for progress update: {}", message.as_ref());
    }
  }

  pub fn finished<S>(&self, message: S)
  where
    S: AsRef<str>,
  {
    let update = ProgressUpdate::Finished {
      text: message.as_ref().to_owned(),
      seq: self.increment_seq_num(),
    };
    let _ = self.tx.send(update);
  }

  fn increment_seq_num(&self) -> usize {
    self
      .sequence_number
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  }
}
