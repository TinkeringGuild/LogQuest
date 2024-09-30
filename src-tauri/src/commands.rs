use crate::{
  common::{
    file_path_is_executable, format_integer, progress_reporter::ProgressUpdate,
    security::is_crypto_available, UUID,
  },
  gina::{importer::import_from_gina_export_file, regex::RegexGINA},
  logs::active_character_detection::Character,
  matchers::MatchContext,
  reactor::ReactorEvent,
  state::{
    config::LogQuestConfig,
    state_handle::StateHandle,
    state_tree::{OverlayState, ReactorState},
    timer_manager::TimerLifetime,
  },
  triggers::{
    command_template::{CommandTemplate, CommandTemplateSecurityCheck},
    trigger_index::{DataDelta, Mutation, TriggerIndex},
  },
  ui::{
    OverlayManagerState, OVERLAY_WINDOW_LABEL, PROGRESS_UPDATE_EVENT_NAME,
    PROGRESS_UPDATE_FINISHED_EVENT_NAME,
  },
};
use serde::Serialize;
use std::{collections::HashSet, path::PathBuf};
use tauri::{AppHandle, Manager, State, Window};
use tokio::sync::mpsc;
use tracing::{debug, error, event, info};

pub const CROSS_DISPATCH_EVENT_NAME: &str = "cross-dispatch";

#[derive(Serialize, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum SystemCommandInfo {
  Executable(String),
  NotExecutable(String),
  NotFound,
}

#[derive(Serialize, ts_rs::TS)]
pub struct Bootstrap {
  config: LogQuestConfig,
  overlay: OverlayState,
  triggers: TriggerIndex,
  reactor: ReactorState,
}

impl Bootstrap {
  fn from_state(state: &StateHandle) -> Self {
    let config = state.select_config(|c| c.clone());
    let triggers = state.select_triggers(|r| r.clone());
    let overlay = state.select_overlay(|o| o.clone());
    let reactor = state.select_reactor(|r| r.clone());

    Self {
      overlay,
      triggers,
      config,
      reactor,
    }
  }
}

pub fn handler() -> impl Fn(tauri::Invoke) {
  tauri::generate_handler![
    bootstrap,
    bootstrap_overlay,
    dispatch_to_overlay,
    get_active_trigger_tags,
    get_config,
    get_current_character,
    import_gina_triggers_file,
    mutate,
    play_audio_file,
    print_to_stderr,
    print_to_stdout,
    set_everquest_dir,
    set_overlay_opacity,
    set_trigger_tag_activated,
    sign_command_template,
    start_timers_sync,
    sys_command_info,
    validate_gina_regex,
    validate_gina_regex_with_context
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
fn get_active_trigger_tags(state: State<StateHandle>) -> Result<HashSet<UUID>, String> {
  let active = state.select_reactor(|c| c.active_trigger_tags.clone());
  Ok(active)
}

#[tauri::command]
fn set_trigger_tag_activated(
  id: UUID,
  activated: bool,
  state: State<StateHandle>,
) -> HashSet<UUID> {
  state.update_reactor_and_select(|c| {
    if activated {
      c.active_trigger_tags.insert(id);
    } else {
      c.active_trigger_tags.remove(&id);
    }
    c.active_trigger_tags.clone()
  })
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

#[tauri::command]
fn sign_command_template(cmd_tmpl: CommandTemplate) -> CommandTemplateSecurityCheck {
  if is_crypto_available() {
    cmd_tmpl.approve()
  } else {
    CommandTemplateSecurityCheck::Unapproved(cmd_tmpl)
  }
}

#[tauri::command]
fn sys_command_info(command: String) -> Result<SystemCommandInfo, String> {
  let as_path = PathBuf::from(&command);
  if as_path.is_absolute() {
    if !as_path.is_file() {
      return Ok(SystemCommandInfo::NotFound);
    }
    if file_path_is_executable(&command) {
      Ok(SystemCommandInfo::Executable(command))
    } else {
      Ok(SystemCommandInfo::NotExecutable(command))
    }
  } else {
    match which::which(command) {
      Ok(path) => Ok(SystemCommandInfo::Executable(path.display().to_string())),
      Err(which::Error::CannotFindBinaryPath) => Ok(SystemCommandInfo::NotFound),
      Err(e) => Err(e.to_string()),
    }
  }
}

#[tauri::command]
fn validate_gina_regex(pattern: String) -> Option<(Option<usize>, String)> {
  match RegexGINA::from_str_without_fixing_character_classes(&pattern) {
    Err(fancy_regex::Error::ParseError(position, parse_error)) => {
      Some((Some(position), parse_error.to_string()))
    }
    Err(fancy_regex::Error::CompileError(compile_error)) => Some((None, compile_error.to_string())),
    _ => None,
  }
}

#[tauri::command]
fn validate_gina_regex_with_context(pattern: String) -> Option<(Option<usize>, String)> {
  match RegexGINA::from_str_with_context_without_fixing_character_classes(
    &pattern,
    &MatchContext::empty(""),
  ) {
    Err(fancy_regex::Error::ParseError(position, parse_error)) => {
      Some((Some(position), parse_error.to_string()))
    }
    Err(fancy_regex::Error::CompileError(compile_error)) => Some((None, compile_error.to_string())),
    _ => None,
  }
}

#[tauri::command]
async fn play_audio_file(
  path: String,
  state: State<'_, mpsc::Sender<ReactorEvent>>,
) -> Result<(), String> {
  state
    .send(ReactorEvent::TestAudioFile(path))
    .await
    .map_err(|_| "Reactor not running".to_owned())
}

#[tauri::command]
fn get_current_character(state: State<StateHandle>) -> Option<Character> {
  state.select_reactor(|reactor| reactor.current_character.clone())
}
