use arboard::Clipboard;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{error, warn};

#[derive(Clone)]
pub struct ClipboardWriter(Result<Arc<AsyncMutex<Clipboard>>, String>);

impl ClipboardWriter {
  pub fn new() -> Self {
    let clipboard_maybe = match Clipboard::new() {
      Ok(clipboard) => Ok(Arc::new(AsyncMutex::new(clipboard))),
      Err(e) => {
        error!("Clipboard writing will be unavailable due to error: {e:?}");
        Err(e.to_string())
      }
    };
    Self(clipboard_maybe)
  }

  pub async fn write_text(&self, text: &str) {
    match &self.0 {
      Ok(clipboard_lock) => {
        let mut guard = clipboard_lock.lock().await;
        if let Err(e) = guard.set_text(text) {
          error!("Could not write text to clipboard due to error: {e:?}");
        }
      }
      Err(init_err_msg) => {
        warn!(
          "Clipboard writing is disabled due to initialization error ({init_err_msg}). Could not write text to clipboard: `{text}`"
        );
      }
    }
  }
}

impl std::fmt::Debug for ClipboardWriter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0.is_ok() {
      f.write_str("AvailableClipboardWriter")
    } else {
      f.write_str("UnavailableClipboardWriter")
    }
  }
}
