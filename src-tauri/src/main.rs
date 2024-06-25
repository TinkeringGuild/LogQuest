// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::{App, AppHandle, GlobalShortcutManager, Manager, WindowBuilder};

struct OverlayState {
    overlay_editable: Mutex<bool>,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            overlay_editable: Mutex::new(false),
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(OverlayState::default())
        .setup(|app| {
            let overlay_window = overlay_window(app).build().unwrap();
            let is_editable = *app.state::<OverlayState>().overlay_editable.lock().unwrap();
            overlay_window
                .set_ignore_cursor_events(!is_editable)
                .expect("Failed to set_ignore_cursor_events");

            // overlay_window.open_devtools();

            let callback_app_handle = app.handle();
            app.global_shortcut_manager()
                .register("CommandOrControl+Alt+Shift+L", move || {
                    toggle_overlay_editable(callback_app_handle.app_handle())
                })
                .expect("Failed registering a global shortcut");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![print_to_console])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_overlay_editable(handle: AppHandle) {
    let state = handle.state::<OverlayState>();
    let mut editable_guard = state.overlay_editable.lock().unwrap();
    let inverse = !*editable_guard;
    *editable_guard = inverse;

    let overlay_window = handle.get_window("overlay").unwrap();
    let _ = overlay_window.emit("editable-changed", inverse);
    overlay_window
        .set_ignore_cursor_events(!inverse)
        .expect("Could not set_ignore_cursor_events!");
    if inverse {
        println!("Overlay editing ENABLED");
    } else {
        println!("Overlay editing DISABLED");
    }
}

fn overlay_window(app: &mut App) -> WindowBuilder {
    tauri::WindowBuilder::new(app, "overlay", tauri::WindowUrl::App("overlay.html".into()))
        .title("LogQuest Overlay")
        .transparent(true)
        .decorations(false)
        .focused(true)
        .fullscreen(true)
        .always_on_top(true)
        .skip_taskbar(true)
}

#[tauri::command]
fn print_to_console(message: String) {
    println!("[UI] {}", message);
}
