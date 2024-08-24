use super::timer_manager::{TimerLifetime, TimerManager, TimerStateUpdate};
use crate::common::shutdown::quitter;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};
use tauri::{async_runtime::spawn, AppHandle, Manager};
use tokio::{
  select,
  sync::{broadcast, oneshot},
};
use tracing::{debug, error};

pub const OVERLAY_MESSAGE_EVENT_NAME: &str = "show-message";
pub const OVERLAY_STATE_UPDATE_EVENT_NAME: &str = "timer-state-update";
pub const OVERLAY_EDITABLE_CHANGED_EVENT_NAME: &str = "editable-changed";

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, clap::ValueEnum)]
pub enum OverlayMode {
  /// Shows the overlay as a transparent fullscreen frameless click-through window
  Default,
  /// Shows the overlay in a normal application window (non-fullscreen)
  Windowed,
  /// Do not create any overlay window
  None,
}

#[derive(Debug)]
pub struct OverlayManager {
  app: AppHandle,
  timer_manager: Arc<TimerManager>,
  emitters: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl OverlayManager {
  pub fn new(app: AppHandle, timer_manager: Arc<TimerManager>) -> Self {
    Self {
      app,
      timer_manager,
      emitters: Mutex::new(HashMap::new()),
    }
  }

  pub async fn start_emitter(&self, window_label: &str) -> Vec<TimerLifetime> {
    let (live_timers, timer_state_updates_subscription) = self.timer_manager.subscribe().await;
    let (tx_stop, rx_stop) = oneshot::channel::<()>();
    spawn(emitter_loop(
      self.app.clone(),
      window_label.to_owned(),
      timer_state_updates_subscription,
      rx_stop,
    ));

    let mut emitters = self
      .emitters
      .lock()
      .expect("OverlayManager emitters mutex poisoned");
    if let Some(replaced_stopper) = emitters.insert(window_label.to_owned(), tx_stop) {
      let _ = replaced_stopper.send(());
    }

    live_timers
  }

  pub fn message(&self, message: String) {
    let _ = self.app.emit_all(OVERLAY_MESSAGE_EVENT_NAME, message);
  }
}

async fn emitter_loop(
  app: AppHandle,
  window_label: String,
  mut timer_state_updates: broadcast::Receiver<TimerStateUpdate>,
  mut rx_stop: oneshot::Receiver<()>,
) {
  debug!("OverlayManager event loop starting...");
  let mut quit = quitter();
  loop {
    select! {
      _ = &mut quit => break,
      _ = &mut rx_stop => break,
      update = timer_state_updates.recv() =>  match update {
        Ok(update) => emit_to_window(&app, &window_label, OVERLAY_STATE_UPDATE_EVENT_NAME, update),
        Err(_recv_error) => break,
      }
    }
  }
  debug!("OverlayManager event loop finished");
}

fn emit_to_window<S>(app: &AppHandle, window_label: &str, event_name: &str, event: S)
where
  S: Serialize + Clone + std::fmt::Debug,
{
  debug!("Sending `{event_name}` event to the the `{window_label}` window: {event:?}");
  if let Err(e) = app.emit_to(window_label, event_name, event) {
    error!("Tried to emit an overlay event to the frontend, but received error: {e:?}");
  }
}
