use super::{
  log_event_broadcaster::NotifyError, log_file_cursor::LogFileCursor, Line, LogFileEvent,
};
use crate::common::shutdown::quitter;
use std::path::Path;
use std::{
  pin::Pin,
  sync::Arc,
  task::{Poll, Waker},
};
use tokio::{
  io::{AsyncBufReadExt as _, AsyncSeekExt as _},
  spawn,
  sync::broadcast,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

pub struct LogLineStream {
  pub cursor: LogFileCursor,
  reader: tokio_stream::wrappers::LinesStream<tokio::io::BufReader<tokio::fs::File>>,
  cancel_token: CancellationToken,
  waker_maybe: Arc<tokio::sync::Mutex<Option<Waker>>>,
}

impl LogLineStream {
  // #[cfg(unix)]
  // fn platform_specific_open_options() -> tokio::fs::OpenOptions {
  //   let mut opts = tokio::fs::OpenOptions::new();
  //   opts.read(true);
  //   opts
  // }

  // #[cfg(windows)]
  // fn platform_specific_open_options() -> tokio::fs::OpenOptions {
  //   use winapi::um::winnt::{FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE};

  //   let mut opts = tokio::fs::OpenOptions::new();
  //   opts
  //     .read(true)
  //     .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE);
  //   opts
  // }

  #[cfg(unix)]
  async fn platform_specific_open<P>(path: P) -> tokio::io::Result<tokio::fs::File>
  where
    P: AsRef<Path>,
  {
    tokio::fs::OpenOptions::new().read(true).open(path).await
  }

  #[cfg(windows)]
  async fn platform_specific_open<P>(path: P) -> tokio::io::Result<tokio::fs::File>
  where
    P: AsRef<Path>,
  {
    // TODO remove winapi Cargo dependency
    // use winapi::um::winnt::{FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE};

    // Reference: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfile2
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    const FILE_SHARE_DELETE: u32 = 0x4;
    const SHARE_MODE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    tokio::fs::OpenOptions::new()
      .read(true)
      .share_mode(SHARE_MODE)
      .open(path)
      .await
  }

  pub async fn create(
    cursor: &LogFileCursor,
    rx_log_file_events: broadcast::Receiver<Result<LogFileEvent, NotifyError>>,
  ) -> tokio::io::Result<Self> {
    let file = Self::platform_specific_open(&cursor.path).await?;
    let mut reader = tokio::io::BufReader::new(file);

    debug!(
      "LogLineStream seeking to position {} for file {}",
      cursor.position, cursor.path
    );
    reader
      .seek(std::io::SeekFrom::Start(cursor.position))
      .await?;

    let reader = tokio_stream::wrappers::LinesStream::new(reader.lines());

    let waker_maybe = Arc::new(tokio::sync::Mutex::new(None::<Waker>));

    let cancel_token = CancellationToken::new();

    tokio::spawn(Self::wake_when_modified(
      cursor.path.clone(),
      waker_maybe.clone(),
      cancel_token.clone(),
      rx_log_file_events,
    ));

    Ok(Self {
      cursor: cursor.to_owned(),
      reader,
      waker_maybe,
      cancel_token,
    })
  }

  async fn wake_when_modified(
    followed_file: String,
    waker: Arc<tokio::sync::Mutex<Option<Waker>>>,
    cancel_token: CancellationToken,
    mut subscription: broadcast::Receiver<Result<LogFileEvent, NotifyError>>,
  ) {
    let wake_waiting_waker = || async {
      let mut guard = waker.lock().await;
      if let Some(waker) = guard.take() {
        waker.wake();
      }
    };

    let mut quit = quitter();
    loop {
      tokio::select! {
        () = &mut quit => {
          debug!("LogLineStream QUITTING for file {followed_file}");
          wake_waiting_waker().await;
          break;
        }
        () = cancel_token.cancelled() => {
          debug!("LogLineStream cancelled for file: {followed_file}");
          wake_waiting_waker().await;
          break;
        }
        log_file_event = subscription.recv() => match log_file_event {
          Ok(Ok(
            LogFileEvent::Updated(event_file) |
            LogFileEvent::Created(event_file)
          )) if event_file == followed_file => {
            debug!("Waking LogLineStream due to modify event for file: {event_file:?}");
            wake_waiting_waker().await;
          }
          Ok(Ok(LogFileEvent::Deleted(event_file))) if event_file == followed_file => {
            error!("Log file got deleted while it was being watched: {followed_file}");
            break;
          }
          Ok(Err(_notify_error)) => {
            error!("LogLineReader terminating due to a NotifyError while watching file: {followed_file}");
            break;
          }
          Err(_recv_error) => {
            debug!("LogLineReader terminating due to the LogEventBroadcaster channel becoming closed");
            break;
          }
          Ok(Ok(_unrelated_file_event)) => {
            debug!("LogLineReader ignoring filesystem event for unrelated file");
            // Got an event for an unrelated file. Nothing to do here...
          }
        }
      }
    }
    debug!("LogLineStream finished for file: {followed_file}");
  }
}

impl futures::Stream for LogLineStream {
  type Item = (Line, LogFileCursor);

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    loop {
      if self.cancel_token.is_cancelled() {
        debug!("LogLineStream cancelled for file {}", self.cursor.path);
        return Poll::Ready(None);
      }
      match Pin::new(&mut self.reader).poll_next(cx) {
        Poll::Pending => return Poll::Pending,
        Poll::Ready(Some(Err(io_error))) => {
          error!(
            "Terminating LogLineStream due to IO error while polling {} [ ERROR = {io_error:?} ]",
            self.cursor.path
          );
          return Poll::Ready(None);
        }
        Poll::Ready(Some(Ok(line))) => {
          let line_len = line.len();
          self.cursor.position += 1 + line_len as u64;

          let line: &str = if line.ends_with("\r") {
            self.cursor.position += 1;
            &line[..(line_len - 1)]
          } else {
            &line
          };

          if let Ok(parsed_line) = Line::from(line) {
            return Poll::Ready(Some((parsed_line, self.cursor.clone())));
          } else {
            // line failed to parse; drop the data and continue with loop
          }
        }
        Poll::Ready(None) => {
          // EOF reached; let a notify event wake up this future later
          let waker = cx.waker().to_owned();
          let wake_state = self.waker_maybe.clone();
          spawn(async move {
            let mut guard = wake_state.lock().await;
            *guard = Some(waker);
          });
          return Poll::Pending;
        }
      }
    }
  }
}

impl Drop for LogLineStream {
  fn drop(&mut self) {
    debug!("LogLineStream dropped for file {}", self.cursor.path);
    self.cancel_token.cancel();
  }
}
