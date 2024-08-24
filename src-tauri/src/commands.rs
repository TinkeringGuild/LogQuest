use crate::{
  common::{format_integer, progress_reporter::ProgressUpdate},
  gina::importer::GINAImport,
  state::{
    config::LogQuestConfig, state_handle::StateHandle, state_tree::OverlayState,
    timer_manager::TimerLifetime,
  },
  triggers::TriggerRoot,
  ui::{OverlayManagerState, PROGRESS_UPDATE_EVENT_NAME, PROGRESS_UPDATE_FINISHED_EVENT_NAME},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, State, Window};
use tracing::{debug, error, event, info};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize)]
pub struct Bootstrap {
  config: LogQuestConfig,
  overlay: OverlayState,
  triggers: TriggerRoot,
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
    get_config,
    import_gina_triggers_file,
    print_to_stderr,
    print_to_stdout,
    set_everquest_dir,
    start_sync,
  ]
}

#[tauri::command]
async fn bootstrap(state: State<'_, StateHandle>) -> Result<Bootstrap, String> {
  Ok(Bootstrap::from_state(&state))
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
async fn start_sync(
  window: Window,
  overlay_manager: State<'_, OverlayManagerState>,
) -> Result<Vec<TimerLifetime>, ()> {
  let window_label = window.label();
  Ok(overlay_manager.start_emitter(window_label).await)
}

#[tauri::command]
async fn import_gina_triggers_file(
  window: Window,
  state: State<'_, StateHandle>,
  path: String,
) -> Result<TriggerRoot, String> {
  let path: PathBuf = path.into();
  let (progress_reporter, watch_progress_updates, rx_gina_import) = GINAImport::load(&path);

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
        let _ = window.emit(event_name, current);
      }
    } else {
      error!("Could not send updates to unknown window: {window_label}");
    }
    debug!("Progress update sender task complete");
  });

  progress_reporter.update("Starting import...");

  let gina_import = match rx_gina_import.await {
    Ok(Ok(import)) => import,
    Ok(Err(import_error)) => return Err(import_error.to_string()),
    Err(_recv_error) => return Err("Import crashed!".to_owned()),
  };

  progress_reporter.update("Adding everything to the Trigger Tree");
  state.update_triggers(move |trigger_root| trigger_root.ingest_gina_import(gina_import));

  let (count_imported, trigger_root) =
    state.select_triggers(|root| (root.trigger_count() - count_before, root.clone()));

  let count_imported = format_integer(count_imported);
  progress_reporter.finished(format!("Added {} Triggers", count_imported));
  info!(
    "Imported {count_imported} new triggers from GINA file: {}",
    path.display()
  );

  Ok(trigger_root)
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
