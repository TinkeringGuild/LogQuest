// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gina;

use std::{path::PathBuf, sync::Mutex};

use clap::{arg, command, value_parser, Arg};
use config::LogQuestConfig;
use gina::load_gina_triggers_from_file_path;
use std::fs::canonicalize;
use tauri::{App, AppHandle, GlobalShortcutManager, Manager, WindowBuilder};

struct AppState {
    overlay_state: OverlayState,
    config: Mutex<LogQuestConfig>,
}

struct OverlayState {
    overlay_editable: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            overlay_state: OverlayState::default(),
            config: Mutex::new(LogQuestConfig::default()),
        }
    }
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            overlay_editable: Mutex::new(false),
        }
    }
}

fn main() {
    init_using_cli_params().expect("Could not initialize!");
}

fn init_using_cli_params() -> anyhow::Result<()> {
    let matches = command!("lq")
        .version("0.1.0")
        .author("Tinkering Guild")
        .about("EverQuest log parser, overlay, notification system, and Deluxe Toolbox for EQ-related assistance")
        .arg(
            Arg::new("config-dir")
                .short('C')
                .long("config-dir")
                .value_name("DIR")
                .help("Specify a specific directory to load LogQuest configs/state from")
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .global(true),
        )
        .subcommand_required(false)
        .subcommand(
            command!("import")
                .about("Import a file")
                .arg(arg!(<FILE>).required(true).index(1).help("Path of the GINA triggers file to import")
                     .value_parser(value_parser!(PathBuf))),
        )
        .get_matches();

    let overridden_config_path = matches.get_one::<PathBuf>("config-dir");
    let config = match config::load_app_config(overridden_config_path.cloned()) {
        Ok(config) => config,
        Err(e) => {
            panic!("Could not load config!\n{:#?}", e);
        }
    };

    if let Some(import_match) = matches.subcommand_matches("import") {
        let Some(file_path) = import_match.get_one::<PathBuf>("FILE") else {
            panic!("No file path given to import?");
        };
        let gina_triggers = load_gina_triggers_from_file_path(file_path.to_owned());
        println!("{:#?}", gina_triggers);
    } else {
        let app_state = load_app_state_from_config(config)?;
        start_ui(app_state);
    }
    Ok(())
}

fn load_app_state_from_config(app_config: LogQuestConfig) -> anyhow::Result<AppState> {
    let state = AppState {
        overlay_state: OverlayState::default(),
        config: Mutex::new(app_config),
    };
    Ok(state)
}

fn start_ui(app_state: AppState) {
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
            app.global_shortcut_manager()
                .register("CommandOrControl+Alt+Shift+L", move || {
                    toggle_overlay_editable(callback_app_handle.app_handle())
                })
                .expect("Failed registering a global shortcut");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            print_to_stdout,
            print_to_stderr,
            get_config,
            set_everquest_dir
        ])
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
        println!("Overlay editing ENABLED");
    } else {
        println!("Overlay editing DISABLED");
    }
}

fn overlay_window_builder(app: &mut App) -> WindowBuilder {
    tauri::WindowBuilder::new(app, "overlay", tauri::WindowUrl::App("overlay.html".into()))
        .title("LogQuest Overlay")
        .transparent(true)
        .decorations(false)
    // .focused(true)
    // .fullscreen(true)
    // .always_on_top(true)
    // .skip_taskbar(true)
}

#[tauri::command]
fn get_config(app_handle: tauri::AppHandle) -> Result<LogQuestConfig, String> {
    let app_state = app_handle.state::<AppState>();
    let config = app_state.config.lock().unwrap().clone();
    Ok(config)
}

#[tauri::command]
fn set_everquest_dir(app_handle: tauri::AppHandle, new_dir: String) -> Result<String, String> {
    println!("SETTING NEW DIR VIA COMMAND: {}", new_dir);
    let app_state = app_handle.state::<AppState>();
    let mut config_guard = app_state
        .config
        .lock()
        .expect("Could not obtain lock for the LogQuestConfig");
    let Ok(new_dir) = canonicalize(new_dir) else {
        return Err("Could not determine canonical path of EQ dir".to_string());
    };
    let new_dir = new_dir.to_str().unwrap();
    let new_dir = new_dir.to_string();
    config_guard.everquest_directory = Some(new_dir.clone());
    Ok(new_dir)
}

#[tauri::command]
fn print_to_stdout(message: String) {
    println!("[UI] {}", message);
}

#[tauri::command]
fn print_to_stderr(message: String) {
    eprintln!("[UI ERROR] {}", message);
}
