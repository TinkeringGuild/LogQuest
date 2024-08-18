use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::task::Poll;
use std::task::Waker;
use tracing::{debug, info};

// TODO: Use LazyLock here
lazy_static! {
  pub static ref IS_SHUTDOWN: AtomicBool = AtomicBool::new(false);
  static ref WAKERS: Mutex<Option<IndexMap<u64, Waker>>> = Mutex::new(Some(IndexMap::new()));
  static ref WAKER_SEQ: AtomicU64 = AtomicU64::new(1);
  static ref CRITICAL_PATH_LOCK: Mutex<()> = Mutex::new(());
}

pub fn shutdown() {
  info!("Quit requested - stopping LogQuest gracefully");

  #[allow(unused)]
  let critical_path = CRITICAL_PATH_LOCK.lock().expect("CRITICAL PATH POISONED");
  let mut guard = WAKERS.lock().expect("WAKERS POISONED");

  if let Some(wakers) = guard.take() {
    IS_SHUTDOWN.store(true, Ordering::SeqCst);
    debug!("Waking all ShutdownFutures in reverse order from creation");
    for (seq_id, waker) in wakers.into_iter().rev() {
      debug!("WAKING ShutdownFuture #{seq_id}");
      waker.wake();
    }
  }
}

pub fn quitter() -> ShutdownFuture {
  ShutdownFuture::new()
}

pub fn critical_path<F, R>(func: F) -> R
where
  F: FnOnce() -> R,
{
  #[allow(unused)]
  let guard = CRITICAL_PATH_LOCK.lock().expect("CRITICAL PATH POISONED");
  func()
}

#[derive(Debug)]
pub struct ShutdownFuture(u64);

impl ShutdownFuture {
  fn new() -> Self {
    let id = WAKER_SEQ.fetch_add(1, Ordering::SeqCst);
    debug!("ShutdownFuture #{id} created");
    Self(id)
  }
}

impl Future for ShutdownFuture {
  type Output = ();

  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    if IS_SHUTDOWN.load(Ordering::SeqCst) {
      Poll::Ready(())
    } else {
      let mut guard = WAKERS.lock().expect("WAKERS POISONED");
      if let Some(map) = &mut *guard {
        map.insert(self.0, cx.waker().clone());
        Poll::Pending
      } else {
        Poll::Ready(())
      }
    }
  }
}

impl Drop for ShutdownFuture {
  fn drop(&mut self) {
    let mut guard = WAKERS.lock().expect("WAKERS POISONED");
    if let Some(wakers) = &mut *guard {
      wakers.shift_remove(&self.0);
    }
  }
}
