use crate::AppState;
use crate::{common::path_string, config::LogQuestConfig};
use anyhow::bail;
use std::{fs::canonicalize, path::Path};
use tauri::{AppHandle, Manager};
use tracing::{event, info};

pub fn handler() -> impl Fn(tauri::Invoke) {
  tauri::generate_handler![
    print_to_stdout,
    print_to_stderr,
    get_config,
    set_everquest_dir,
    import_gina_triggers_file
  ]
}

#[tauri::command]
fn get_config(app_handle: AppHandle) -> Result<LogQuestConfig, String> {
  let app_state = app_handle.state::<AppState>();
  let config = app_state.config.lock().unwrap().clone();
  Ok(config)
}

#[tauri::command]
fn import_gina_triggers_file(
  app_handle: AppHandle,
  _path: String,
) -> Result<LogQuestConfig, String> {
  let app_state = app_handle.state::<AppState>();
  let config = app_state.config.lock().unwrap().clone();
  Ok(config)
}

#[tauri::command]
fn set_everquest_dir(app_handle: AppHandle, new_dir: String) -> Result<LogQuestConfig, String> {
  let Ok(new_dir) = canonicalize(new_dir) else {
    return Err("Could not determine canonical path of EQ dir".to_owned());
  };
  validate_eq_dir(&new_dir).map_err(|e| e.to_string())?;
  with_config(&app_handle, |config| {
    config.everquest_directory = Some(path_string(&new_dir));
    Ok(())
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

/// Yields a mutable borrow to the LogQuestConfig and automatically saves the file if any changes are made.
fn with_config<F>(app_handle: &AppHandle, f: F) -> Result<LogQuestConfig, String>
where
  F: FnOnce(&mut LogQuestConfig) -> Result<(), String>,
{
  let app_state = app_handle.state::<AppState>();
  let mut config_guard = app_state
    .config
    .lock()
    .expect("Could not obtain lock for the LogQuestConfig");
  let config_before: LogQuestConfig = config_guard.clone();
  f(&mut *config_guard)?;
  if *config_guard != config_before {
    info!(
      "SAVING CONFIG TO {}",
      config_before.config_file_path.display()
    );
    config_guard.save().map_err(|e| e.to_string())?;
  }
  Ok(config_guard.clone())
}

fn validate_eq_dir(path: &Path) -> anyhow::Result<()> {
  let eqclient_file = path.join("eqclient.ini");
  if eqclient_file.exists() {
    Ok(())
  } else {
    bail!(
      r#"The path "{}" is not a valid EverQuest directory!"#,
      path.display()
    )
  }
}
