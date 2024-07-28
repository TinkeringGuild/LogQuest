use crate::{
  commands,
  common::{fatal_error, ternary},
  state::AppState,
};
use std::sync::{Mutex, MutexGuard};
use tauri::{App, AppHandle, GlobalShortcutManager, Manager, Window, WindowBuilder, WindowEvent};
use tracing::info;

const TOGGLE_OVERLAY_ACCELERATOR: &str = "CommandOrControl+Alt+Shift+L";

pub fn launch(app_state: AppState) {
  let result = tauri::Builder::default()
    .manage(app_state)
    .setup(setup)
    .invoke_handler(commands::handler())
    .run(tauri::generate_context!());

  if let Err(e) = result {
    fatal_error(&e.to_string());
  }
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
  create_overlay_window(app)/*.open_devtools()*/;
  register_window_close_event(app.handle());
  register_global_shortcut_manager(app.handle());
  Ok(())
}

fn overlay_window_builder(app: &mut App) -> WindowBuilder {
  tauri::WindowBuilder::new(app, "overlay", tauri::WindowUrl::App("overlay.html".into()))
    .title("LogQuest Overlay")
    .transparent(true)
    .decorations(false)
    .focused(true)
    .fullscreen(true)
    .always_on_top(true)
    .skip_taskbar(true)
}

// fn select_state<F, T>(app: &mut App, f: F) -> T where F: FnOnce(&AppState) -> T { f(&app.state::<AppState>()) }

fn select_mutex_value<F, T>(app: &mut App, f: F) -> T
where
  F: FnOnce(&AppState) -> &Mutex<T>,
  T: Clone,
{
  let state = app.state::<AppState>();
  let mutex = f(&state);
  let value = mutex.lock().expect("mutex deadlocked?");
  value.clone()
}

fn create_overlay_window(app: &mut App) -> tauri::Window {
  let overlay_window = overlay_window_builder(app).build().unwrap();
  let is_editable = select_mutex_value(app, |state| &state.overlay_state.overlay_editable);
  overlay_window
    .set_ignore_cursor_events(!is_editable)
    .expect("Failed to set_ignore_cursor_events");
  overlay_window
}

fn get_main_window(app: AppHandle) -> Window {
  app
    .get_window("main")
    .expect("Expected main window to exist!")
}

fn register_window_close_event(app: AppHandle) {
  let handle = app.app_handle();
  let window = get_main_window(app);
  window.on_window_event(move |window_event: &WindowEvent| match window_event {
    WindowEvent::Destroyed => {
      handle.exit(0);
    }
    _ => {}
  });
}

fn register_global_shortcut_manager(app: AppHandle) {
  app
    .global_shortcut_manager()
    .register(TOGGLE_OVERLAY_ACCELERATOR, move || {
      toggle_overlay_editable(app.app_handle())
    })
    .expect("Failed registering a global shortcut!");
}

fn set_locked_state_value<S, T, E>(app: AppHandle, selector: S, edit: E)
where
  S: FnOnce(&AppState) -> &Mutex<T>,
  E: FnOnce(&MutexGuard<T>) -> T,
{
  let state = app.state::<AppState>();
  let mutex = selector(&state);
  let mut lock = mutex.lock().expect("mutex deadlocked?");
  let new_value = edit(&lock);
  *lock = new_value;
}

fn toggle_overlay_editable(app: AppHandle) {
  let handle = app.app_handle();
  set_locked_state_value(
    app,
    |state: &AppState| &state.overlay_state.overlay_editable,
    move |overlay_editable| {
      let inverse = !**overlay_editable;
      if let Some(overlay_window) = handle.get_window("overlay") {
        let _ = overlay_window.emit("editable-changed", inverse);

        overlay_window
          .set_ignore_cursor_events(!inverse) // set_ignore_cursor_events takes a bool opposite of how it's stored in AppState
          .expect("Could not set_ignore_cursor_events!");

        info!(
          "Overlay editing {}",
          ternary(inverse, "ENABLED", "DISABLED")
        )
      }
      inverse
    },
  );
}
