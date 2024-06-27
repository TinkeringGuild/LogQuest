// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gina;

use std::{path::PathBuf, sync::Mutex};

use clap::{arg, command, value_parser};
use gina::load_gina_triggers_from_file_path;
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
    parse_cli_params();
}

fn parse_cli_params() {
    let matches = command!("lq")
        .version("0.1.0")
        .author("Tinkering Guild")
        .about("EverQuest log parser, overlay, notification system, and Deluxe Toolbox for EQ-related assistance")
        .subcommand_required(false)
        .subcommand(
            command!("import")
                .about("Import a file")
                .arg(arg!(<FILE>).required(true).index(1).help("Path of the GINA triggers file to import")
                     .value_parser(value_parser!(PathBuf))),
        )
        .get_matches();

    if let Some(import_match) = matches.subcommand_matches("import") {
        let Some(file_path) = import_match.get_one::<PathBuf>("FILE") else {
            panic!("No file path given to import?");
        };
        let gina_triggers = load_gina_triggers_from_file_path(file_path.to_owned());
        println!("{:#?}", gina_triggers);
    } else {
        start_ui();
    }
}

fn start_ui() {
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
