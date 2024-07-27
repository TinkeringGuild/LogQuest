use tauri::{App, AppHandle, GlobalShortcutManager, Manager, WindowBuilder, WindowEvent};
use tracing::info;

use crate::{commands, state::AppState};

pub fn start_ui(app_state: AppState) {
  tauri::Builder::default()
    .manage(app_state)
    .setup(|app| {
      let overlay_window = overlay_window_builder(app).build().unwrap();
      let is_editable = *app
        .state::<AppState>()
        .overlay_state
        .overlay_editable
        .lock()
        .expect("overlay_editable appears deadlocked!");

      overlay_window
        .set_ignore_cursor_events(!is_editable)
        .expect("Failed to set_ignore_cursor_events");

      // overlay_window.open_devtools();

      let callback_app_handle = app.handle();
      app
        .get_window("main")
        .unwrap()
        .on_window_event(move |window_event: &WindowEvent| match window_event {
          WindowEvent::Destroyed => {
            callback_app_handle.exit(0);
          }
          _ => {}
        });

      let callback_app_handle = app.handle();
      app
        .global_shortcut_manager()
        .register("CommandOrControl+Alt+Shift+L", move || {
          toggle_overlay_editable(callback_app_handle.app_handle())
        })
        .expect("Failed registering a global shortcut");
      Ok(())
    })
    .invoke_handler(commands::handler())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn toggle_overlay_editable(handle: AppHandle) {
  let state = handle.state::<AppState>();
  let mut editable_guard = state.overlay_state.overlay_editable.lock().unwrap();
  let inverse = !*editable_guard;
  *editable_guard = inverse;

  let overlay_window = handle.get_window("overlay").unwrap();
  let _ = overlay_window.emit("editable-changed", inverse);
  overlay_window
    .set_ignore_cursor_events(!inverse)
    .expect("Could not set_ignore_cursor_events!");
  if inverse {
    info!("Overlay editing ENABLED");
  } else {
    info!("Overlay editing DISABLED");
  }
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
