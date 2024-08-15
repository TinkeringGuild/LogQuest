use crate::{
  gina::GINAImport,
  state::{
    config::LogQuestConfig, state_handle::StateHandle, state_tree::OverlayState, timers::LiveTimer,
  },
  triggers::TriggerRoot,
  ui::OverlayManagerState,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{State, Window};
use tracing::{event, info};
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
async fn bootstrap<'a>(state: State<'_, StateHandle>) -> Result<Bootstrap, String> {
  Ok(Bootstrap::from_state(&state))
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
) -> Result<Vec<LiveTimer>, ()> {
  let window_label = window.label();
  Ok(overlay_manager.start_emitter(window_label).await)
}

#[tauri::command]
async fn import_gina_triggers_file(
  state: State<'_, StateHandle>,
  path: String,
) -> Result<TriggerRoot, String> {
  let path: PathBuf = path.into();
  let gina_import = GINAImport::load(&path).map_err(|e| e.to_string())?;

  let count_before = state.select_triggers(|root| root.trigger_count());

  state.update_triggers(move |trigger_root| trigger_root.ingest_gina_import(gina_import));

  let (count_after, trigger_root) =
    state.select_triggers(|root| (root.trigger_count(), root.clone()));

  info!(
    "Imported {} new triggers from GINA file: {}",
    count_after - count_before,
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

#[tauri::command]
fn print_to_stdout(message: String) {
  event!(target: "UI", tracing::Level::INFO, message);
}

#[tauri::command]
fn print_to_stderr(message: String) {
  event!(target: "UI", tracing::Level::ERROR, message);
}
