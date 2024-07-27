// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod active_character_detection;
mod cli;
mod commands;
mod common;
mod config;
mod gina;
mod log_reader;
mod matchers;
mod state;
mod triggers;
mod ui;

#[cfg(debug_assertions)]
mod debug_only;

use cli::{Commands, StartCommand};
use state::AppState;
use std::{path::PathBuf, process::exit};
use tracing::error;

fn init_tracing() {
  let env_filter = tracing_subscriber::EnvFilter::from_default_env();
  tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

fn main() {
  init_tracing();

  let result = match cli::cmd() {
    Commands::Start(StartCommand { config_dir }) => start(config_dir),

    #[cfg(debug_assertions)]
    Commands::TS => debug_only::generate_typescript(),

    #[cfg(debug_assertions)]
    Commands::ConvertGINA { file, format } => debug_only::convert_gina(&file, format),
  };
  if let Err(e) = result {
    fatal_error(&format!("{:?}", e));
  }
}

fn start(config_dir: Option<PathBuf>) -> anyhow::Result<()> {
  let config_dir = config::get_config_dir_with_optional_override(config_dir)?;
  let config = config::load_or_create_app_config_from_dir(&config_dir)?;
  let app_state = AppState::init_from_config(config)?;
  ui::start_ui(app_state);
  Ok(())
}

fn fatal_error(message: &str) {
  error!(message);
  exit(2);
}
