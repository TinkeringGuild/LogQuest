use crate::state::config::LogQuestConfig;
use crate::{common::path_string, state::state_handle::StateHandle};
use anyhow::bail;
use std::{fs::canonicalize, path::Path};
use tauri::{AppHandle, Manager};
use tracing::event;

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
  let state = app_handle.state::<StateHandle>();
  let config = state.select_config(|c| c);
  Ok(config)
}

#[tauri::command]
fn import_gina_triggers_file(
  app_handle: AppHandle,
  _path: String,
) -> Result<LogQuestConfig, String> {
  let state = app_handle.state::<StateHandle>();
  let config = state.select_config(|c| c);
  // TODO: THIS SHOULD NOT RETURN CONFIG BECAUSE TRIGGERS ARE NOT STORED IN
  // CONFIG. THIS IS JUST A NO-OP FOR NOW, ESSENTIALLY.
  Ok(config)
}

#[tauri::command]
fn set_everquest_dir(app_handle: AppHandle, new_dir: String) -> Result<LogQuestConfig, String> {
  let Ok(new_dir) = canonicalize(new_dir) else {
    return Err("Could not determine canonical path of EQ dir".to_owned());
  };
  validate_eq_dir(&new_dir).map_err(|e| e.to_string())?;
  let state = app_handle.state::<StateHandle>();
  // TODO: I should introduce a try_with_config method that allows the callback to return a Result
  // since writing to the file can fail.
  state.with_config(|config| {
    config.everquest_directory = Some(path_string(&new_dir));
  });
  Ok(state.select_config(|c| c))
}

#[tauri::command]
fn print_to_stdout(message: String) {
  event!(target: "UI", tracing::Level::INFO, message);
}

#[tauri::command]
fn print_to_stderr(message: String) {
  event!(target: "UI", tracing::Level::ERROR, message);
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
