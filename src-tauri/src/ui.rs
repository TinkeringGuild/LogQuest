use crate::{
  commands,
  common::{fatal_error, ternary},
  reactor,
  state::state_handle::StateHandle,
};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn;
use tauri::{App, AppHandle, GlobalShortcutManager, Manager, Window, WindowEvent};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, clap::ValueEnum)]
pub enum OverlayMode {
  /// Shows the overlay as a transparent fullscreen frameless click-through window
  Default,
  /// Shows the overlay in a normal application window (non-fullscreen)
  Windowed,
  /// Do not create any overlay window
  None,
}

const TOGGLE_OVERLAY_ACCELERATOR: &str = "CommandOrControl+Alt+Shift+L";

pub fn launch(state: StateHandle) {
  let result = tauri::Builder::default()
    .manage(state.clone())
    .setup(move |app: &mut App| {
      reactor(&state);
      setup(&app.app_handle());
      Ok(())
    })
    .invoke_handler(commands::handler())
    .run(tauri::generate_context!());

  if let Err(e) = result {
    fatal_error(e);
  }
}

fn reactor(state_handle: &StateHandle) {
  let future = reactor::start_when_config_is_ready(state_handle);
  spawn(async move {
    match future.await {
      Ok(Ok(_stop_reactor)) => {
        debug!("Reactor started from UI");
      }
      Err(_recv_error) => {
        fatal_error("Reactor start_when_config_is_ready future channel closed?");
      }
      Ok(Err(reactor_start_error)) => {
        fatal_error(reactor_start_error.to_string());
      }
    }
  });
}

fn setup(app: &AppHandle) {
  register_main_window_close_event(app);
  register_global_shortcut_manager(app);
  setup_overlay(app);
}

fn setup_overlay(app: &AppHandle) {
  let state = app.state::<StateHandle>();
  let overlay_mode = state.select_overlay(|o| o.overlay_mode.clone());
  match overlay_mode {
    OverlayMode::Default => {
      create_default_overlay_window(app);
    }
    OverlayMode::Windowed => {
      create_windowed_overlay_window(app);
    }
    OverlayMode::None => {}
  }
}

fn create_default_overlay_window(app: &AppHandle) -> tauri::Window {
  let window_uri = tauri::WindowUrl::App("overlay.html".into());
  let overlay_window = tauri::WindowBuilder::new(app, "overlay", window_uri)
    .title("LogQuest Overlay")
    .transparent(true)
    .decorations(false)
    .fullscreen(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .build()
    .expect("Could not create overlay window!");

  let state = app.state::<StateHandle>();
  let is_editable = state.select_overlay(|overlay| overlay.overlay_editable);
  set_overlay_editable(app, is_editable);

  overlay_window
}

fn create_windowed_overlay_window(app: &AppHandle) -> tauri::Window {
  let window_uri = tauri::WindowUrl::App("overlay.html".into());
  let overlay_window = tauri::WindowBuilder::new(app, "overlay", window_uri)
    .title("LogQuest Overlay")
    .build()
    .expect("Could not create overlay window!");
  overlay_window
}

fn get_main_window(app: &AppHandle) -> Window {
  // TODO: When the main Window is allowed to be closed, will this cause a panic
  // because of the expect()? or does Tauri keep the Window instance while closed?
  app
    .get_window("main")
    .expect("Expected main window to exist!")
}

fn get_overlay_window(app: &AppHandle) -> Option<Window> {
  let state = app.state::<StateHandle>();
  let overlay_mode = state.select_overlay(|o| o.overlay_mode.clone());
  if let OverlayMode::None = overlay_mode {
    return None;
  }
  app.get_window("overlay")
}

fn register_main_window_close_event(app: &AppHandle) {
  let window = get_main_window(&app);
  let app = app.clone();
  window.on_window_event(move |window_event: &WindowEvent| match window_event {
    WindowEvent::Destroyed => {
      app.exit(0);
    }
    _ => {}
  });
}

fn register_global_shortcut_manager(app: &AppHandle) {
  let app = app.clone();
  app
    .global_shortcut_manager()
    .register(TOGGLE_OVERLAY_ACCELERATOR, move || {
      toggle_overlay_editable(&app)
    })
    .expect("Failed registering a global shortcut!");
}

fn toggle_overlay_editable(app: &AppHandle) {
  let state = app.state::<StateHandle>();
  let inverse = state.select_overlay(|o| !o.overlay_editable);
  set_overlay_editable(app, inverse);
}

fn set_overlay_editable(app: &AppHandle, new_value: bool) {
  let state = app.state::<StateHandle>();
  state.update_overlay(|overlay| overlay.overlay_editable = new_value);
  if let Some(overlay_window) = get_overlay_window(app) {
    overlay_window
      .set_ignore_cursor_events(!new_value)
      .expect("Failed to set_ignore_cursor_events");
    let _ = overlay_window.emit("editable-changed", new_value);
    info!(
      "Overlay editing {}",
      ternary(new_value, "ENABLED", "DISABLED")
    )
  }
}
