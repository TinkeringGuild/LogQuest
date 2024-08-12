use crate::{
  commands,
  common::{fatal_error, ternary},
  reactor,
  state::state_handle::StateHandle,
};
use tauri::async_runtime::spawn;
use tauri::{App, AppHandle, GlobalShortcutManager, Manager, Window, WindowBuilder, WindowEvent};
use tracing::{debug, info};

const TOGGLE_OVERLAY_ACCELERATOR: &str = "CommandOrControl+Alt+Shift+L";

pub fn launch(state: StateHandle) {
  let result = tauri::Builder::default()
    .manage(state.clone())
    .setup(move |app: &mut App| {
      reactor(&state);
      setup(app)
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
      Ok(Err(e)) => {
        fatal_error(e.to_string());
      }
    }
  });
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
  // create_overlay_window(app)/*.open_devtools()*/;
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

fn create_overlay_window(app: &mut App) -> tauri::Window {
  let overlay_window = overlay_window_builder(app).build().unwrap();
  let state = app.state::<StateHandle>();
  let is_editable = state.select_overlay(|overlay| overlay.overlay_editable);
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

fn toggle_overlay_editable(app: AppHandle) {
  let handle = app.app_handle();
  let state = app.state::<StateHandle>();
  state.update_overlay(move |overlay| {
    let inverse = !overlay.overlay_editable;
    overlay.overlay_editable = inverse;
    if let Some(overlay_window) = handle.get_window("overlay") {
      // TODO: Instead of an event, this should send a state change through the reducer
      let _ = overlay_window.emit("editable-changed", inverse);

      overlay_window
        .set_ignore_cursor_events(!inverse) // set_ignore_cursor_events takes a bool opposite of how it's stored in OverlayState
        .expect("Could not set_ignore_cursor_events!");

      info!(
        "Overlay editing {}",
        ternary(inverse, "ENABLED", "DISABLED")
      )
    }
  });
}
