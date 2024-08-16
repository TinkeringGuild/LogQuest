use lazy_static::lazy_static;
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::task::Poll;
use std::task::Waker;

lazy_static! {
  pub static ref IS_SHUTDOWN: AtomicBool = AtomicBool::new(false);
  static ref WAKERS: Mutex<Option<HashMap<u64, Waker>>> = Mutex::new(Some(HashMap::new()));
  static ref WAKER_SEQ: AtomicU64 = AtomicU64::new(0);
}

pub fn shutdown() {
  let mut guard = WAKERS.lock().expect("WAKERS POISONED");
  if let Some(wakers) = guard.take() {
    IS_SHUTDOWN.store(true, Ordering::SeqCst);
    wakers.into_iter().for_each(|(_, waker)| waker.wake());
  }
}

pub fn quitter() -> ShutdownFuture {
  ShutdownFuture::new()
}

pub struct ShutdownFuture(u64);

impl ShutdownFuture {
  fn new() -> Self {
    let id = WAKER_SEQ.fetch_add(1, Ordering::SeqCst);
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
      let waker = cx.waker().clone();
      let mut guard = WAKERS.lock().expect("WAKERS POISONED");
      if let Some(map) = &mut *guard {
        map.insert(self.0.clone(), waker);
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
    if let Some(map) = &mut *guard {
      map.remove(&self.0);
    }
  }
}
