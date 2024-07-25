// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod active_character_detection;
mod commands;
mod common;
mod config;
mod gina;
mod log_reader;
mod matchers;
mod triggers;

use anyhow::bail;
use clap::{arg, command, value_parser, Arg, Command};
use common::timestamp::Timestamp;
use config::LogQuestConfig;
use gina::xml::load_gina_triggers_from_file_path;
use std::{fs, path::PathBuf, process::exit, sync::Mutex};
use tauri::{App, AppHandle, GlobalShortcutManager, Manager, WindowBuilder, WindowEvent};
use tracing::info;
use ts_rs::TS;

struct AppState {
  overlay_state: OverlayState,
  config: Mutex<LogQuestConfig>,
}

struct OverlayState {
  overlay_editable: Mutex<bool>,
}

impl AppState {
  fn init_from_config(app_config: LogQuestConfig) -> anyhow::Result<AppState> {
    let state = AppState {
      overlay_state: OverlayState::default(),
      config: Mutex::new(app_config),
    };
    Ok(state)
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
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  info!("Initializing...");
  if let Err(e) = init_using_cli_params() {
    eprintln!("FATAL ERROR: {:#?}", e);
    exit(1);
  }
}

fn init_using_cli_params() -> anyhow::Result<()> {
  let mut cmd = command!("lq")
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
                .global(true), // TODO: The arg should only be available when no other sub-command is given
        )
        .subcommand_required(false)
        // TODO: convert-gina could probably be a development-only subcommand
        .subcommand(
            Command::new("convert-gina")
                .about("Parse a GINA .gtp or .xml file and output its data")
                .arg(
                    arg!(<FILE>)
                        .required(true)
                        .index(1)
                        .help("Path of the GINA triggers file to import")
                        .value_parser(value_parser!(PathBuf)))
            .arg(
                Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("Specify the format of the output")
                    .value_name("format")
                    .value_parser(["gina-internal", "gina-json", "internal", "json"])
                    .default_value("json")
            )
        );

  #[cfg(debug_assertions)]
  {
    // the "ts" sub-command only exists for debug builds of LogQuest. This allows
    // the file to be written to a specific directory (outside of the cargo root)
    // and avoids having to use "cargo test" to generate the TS files.
    cmd = cmd.subcommand(command!("ts").about("Generate TypeScript from Rust types"));

    // cmd = cmd.subcommand(
    //     command!("watcher").about("Temporary testing tool for filesystem event watcher"),
    // );

    cmd = cmd.subcommand(command!("tail").about("Tails a log file"));
  }

  let matches = cmd.get_matches();

  #[cfg(debug_assertions)]
  {
    if matches.subcommand_matches("ts").is_some() {
      return generate_typescript();
    }

    if matches.subcommand_matches("tail").is_some() {
      let rt = tokio::runtime::Runtime::new().unwrap();
      let log_file_path =
        PathBuf::from("/home/j/code/LogQuest/src-tauri/test_logs/eqlog_Laut_project1999.txt");
      let mut fs_events = log_reader::LogEventBroadcaster::new(&log_file_path)?;
      fs_events.start()?;
      let fs_event_rx = fs_events.subscribe();

      let reader = log_reader::LogReader::start(rt.handle().to_owned(), log_file_path, fs_event_rx);
      let mut rx = reader.subscribe();
      rt.spawn(async move {
        let sleep_secs = 10;
        info!("Sleeping for {sleep_secs} seconds, then stopping the log reader");
        tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
        info!("Stopping log reader now");
        reader.stop();
      });

      rt.block_on(async move {
        loop {
          tokio::select! {
              line = rx.recv() => {
                  info!("{line:#?}");
              }
          }
        }
      });

      return Ok(());
    }
  }

  if let Some(convert_match) = matches.subcommand_matches("convert-gina") {
    let file_path = convert_match.get_one::<PathBuf>("FILE").unwrap();
    let format = convert_match.get_one::<String>("format").unwrap();
    return translate_gina(file_path, &format);
  }

  let overridden_config_path = matches.get_one::<PathBuf>("config-dir");
  let config_path = config::get_config_dir_with_optional_override(overridden_config_path)?;
  let Ok(config) = config::load_app_config(&config_path) else {
    bail!("Could not load config!");
  };

  let app_state = AppState::init_from_config(config)?;
  start_ui(app_state);
  Ok(())
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
    .focused(true)
    .fullscreen(true)
    .always_on_top(true)
    .skip_taskbar(true)
}

fn translate_gina(path: &PathBuf, format: &str) -> anyhow::Result<()> {
  let from_gina = load_gina_triggers_from_file_path(path)?;
  if format == "gina-internal" {
    println!("{from_gina:#?}");
    return Ok(());
  } else if format == "gina-json" {
    match serde_json::to_string_pretty(&from_gina) {
      Ok(raw_json) => {
        println!("{raw_json}");
        return Ok(());
      }
      Err(e) => {
        eprintln!("Failed to serialize GINA types to JSON!");
        bail!(e)
      }
    }
  }
  let root_trigger_group = from_gina.to_lq(&Timestamp::now())?;
  match format {
    "internal" => println!("{root_trigger_group:#?}"),
    "json" => match serde_json::to_string_pretty(&root_trigger_group) {
      Ok(raw_json) => println!("{raw_json}"),
      Err(e) => {
        eprintln!("Failed to serialize to JSON!");
        return Err(e.into());
      }
    },
    _ => bail!("clap should guarantee a valid format parameter"),
  }
  Ok(())
}

#[cfg(debug_assertions)]
fn generate_typescript() -> anyhow::Result<()> {
  let rust_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let out_dir = rust_dir.join("../src/generated/").canonicalize()?;
  if !out_dir.exists() {
    panic!("The src/generated/ dir does not exist!");
  }
  let out_file = out_dir.join("LogQuestConfig.ts");
  if out_file.exists() {
    println!(
      "Deleting possibly stale file {}",
      out_file.to_string_lossy()
    );
    if let Err(e) = fs::remove_file(&out_file) {
      panic!(
        "Could not delete the file {} [ {:?} ]",
        out_file.to_string_lossy(),
        e
      );
    }
  }
  if let Err(e) = LogQuestConfig::export_all_to(&out_dir) {
    panic!("Could not export TypeScript! {:?}", e);
  }

  println!("Exported LogQuestConfig to {}", out_file.to_string_lossy());

  Ok(())
}
