use crate::{
  common::{format_integer, progress_reporter::ProgressUpdate},
  gina::importer::import_from_gina_export_file,
  state::{
    config::LogQuestConfig, state_handle::StateHandle, state_tree::OverlayState,
    timer_manager::TimerLifetime,
  },
  triggers::trigger_index::{DataDelta, Mutation, TriggerIndex},
  ui::{
    OverlayManagerState, OVERLAY_WINDOW_LABEL, PROGRESS_UPDATE_EVENT_NAME,
    PROGRESS_UPDATE_FINISHED_EVENT_NAME,
  },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State, Window};
use tracing::{debug, error, event, info};

pub const CROSS_DISPATCH_EVENT_NAME: &str = "cross-dispatch";

#[derive(Serialize, Deserialize, ts_rs::TS)]
pub struct Bootstrap {
  config: LogQuestConfig,
  overlay: OverlayState,
  triggers: TriggerIndex,
}

impl Bootstrap {
  fn from_state(state: &StateHandle) -> Self {
    let config = state.select_config(|c| c.clone());
    let triggers = state.select_triggers(|r| r.clone());
    let overlay = state.select_overlay(|o| o.clone());
    Self {
      overlay,
      triggers,
      config,
    }
  }
}

pub fn handler() -> impl Fn(tauri::Invoke) {
  tauri::generate_handler![
    bootstrap,
    bootstrap_overlay,
    dispatch_to_overlay,
    get_config,
    import_gina_triggers_file,
    mutate,
    print_to_stderr,
    print_to_stdout,
    set_everquest_dir,
    set_overlay_opacity,
    start_timers_sync,
  ]
}

#[tauri::command]
async fn bootstrap(state: State<'_, StateHandle>) -> Result<Bootstrap, String> {
  Ok(Bootstrap::from_state(&state))
}

#[tauri::command]
fn bootstrap_overlay(state: State<'_, StateHandle>) -> OverlayState {
  state.select_overlay(|o| o.clone())
}

#[tauri::command]
fn print_to_stdout(message: String) {
  event!(target: "UI", tracing::Level::INFO, message);
}

#[tauri::command]
fn print_to_stderr(message: String) {
  event!(target: "UI", tracing::Level::ERROR, message);
}

#[tauri::command]
fn get_config(state: State<StateHandle>) -> Result<LogQuestConfig, String> {
  let config = state.select_config(|c| c.clone());
  Ok(config)
}

#[tauri::command]
fn dispatch_to_overlay(action: serde_json::Value, app: AppHandle) {
  if let Some(overlay_window) = app.get_window(OVERLAY_WINDOW_LABEL) {
    _ = overlay_window.emit(CROSS_DISPATCH_EVENT_NAME, action);
  }
}

#[tauri::command]
fn set_overlay_opacity(opacity: u8, state: State<StateHandle>) {
  state.update_overlay(|o| o.overlay_opacity = opacity);
}

#[tauri::command]
async fn start_timers_sync(
  window: Window,
  overlay_manager: State<'_, OverlayManagerState>,
) -> Result<Vec<TimerLifetime>, ()> {
  let window_label = window.label();
  Ok(overlay_manager.start_emitter(window_label).await)
}

#[tauri::command]
fn mutate(mutations: Vec<Mutation>, state: State<StateHandle>) -> Result<Vec<DataDelta>, String> {
  state
    .mutate_index(|index| {
      mutations
        .into_iter()
        .try_fold(Vec::new(), |mut memo, mutation| {
          memo.append(&mut index.mutate(mutation)?);
          Ok(memo)
        })
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_gina_triggers_file(
  window: Window,
  state: State<'_, StateHandle>,
  path: String,
) -> Result<TriggerIndex, String> {
  let path: PathBuf = path.into();
  let (progress_reporter, watch_progress_updates, rx_gina_import) =
    import_from_gina_export_file(&path, (*state).clone());

  let count_before = state.select_triggers(|root| root.trigger_count());

  let mut watch_progress_updates = Box::pin(watch_progress_updates);
  let window_label = window.label().to_owned();
  let app_handle = window.app_handle();
  tauri::async_runtime::spawn(async move {
    if let Some(window) = app_handle.get_window(&window_label) {
      while let Ok(()) = watch_progress_updates.changed().await {
        let current: &ProgressUpdate = &*watch_progress_updates.borrow();
        let event_name = if let ProgressUpdate::Finished { .. } = current {
          PROGRESS_UPDATE_FINISHED_EVENT_NAME
        } else {
          PROGRESS_UPDATE_EVENT_NAME
        };
        _ = window.emit(event_name, current);
      }
    } else {
      error!("Could not send updates to unknown window: {window_label}");
    }
    debug!("Progress update sender task complete");
  });

  progress_reporter.update("Starting import...");

  match rx_gina_import.await {
    Ok(Ok(())) => {}
    Ok(Err(import_error)) => return Err(import_error.to_string()),
    Err(_recv_error) => return Err("Import crashed!".to_owned()),
  };

  let index_copy = state.select_triggers(|index| index.clone());

  let count_imported = format_integer(index_copy.trigger_count() - count_before);
  progress_reporter.finished(format!("Added {} Triggers", count_imported));
  info!(
    "Imported {count_imported} new triggers from GINA file: {}",
    path.display()
  );

  Ok(index_copy)
}

#[tauri::command]
fn set_everquest_dir(state: State<StateHandle>, new_dir: String) -> Result<LogQuestConfig, String> {
  state.update_config_and_select(|config| {
    config
      .set_eq_dir(&new_dir)
      .map(|_| config.clone())
      .map_err(|e| e.to_string())
  })
}
