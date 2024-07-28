// The next line prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(debug_assertions)]
mod debug_only;

mod cli;
mod commands;
mod common;
mod config;
mod gina;
mod logs;
mod matchers;
mod reactor;
mod state;
mod triggers;
mod ui;

use cli::{Commands, StartCommand};
use common::fatal_error;
use state::AppState;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_LEVEL: &str = "debug";

fn init_tracing() {
  let env_filter = EnvFilter::try_from_default_env();
  let env_filter = env_filter.unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL));
  tracing_subscriber::fmt().with_env_filter(env_filter).init()
}

fn main() {
  init_tracing();

  let result = match cli::cmd() {
    Commands::Start(StartCommand {
      config_dir,
      logs_dir,
    }) => start(config_dir, logs_dir),

    #[cfg(debug_assertions)]
    Commands::TS => debug_only::generate_typescript(),

    #[cfg(debug_assertions)]
    Commands::Tail { file } => debug_only::tail(&file),

    #[cfg(debug_assertions)]
    Commands::ConvertGINA { file, format } => debug_only::convert_gina(&file, format),
  };
  if let Err(e) = result {
    common::fatal_error(&format!("{:?}", e));
  }
}

fn start(
  config_dir_override: Option<PathBuf>,
  logs_dir_override: Option<PathBuf>,
) -> anyhow::Result<()> {
  let config_dir = config::get_config_dir_with_optional_override(config_dir_override)?;
  let config = config::load_or_create_app_config_from_dir(&config_dir)?;

  let logs_dir = logs_dir_override.or_else(|| config.logs_dir());
  let Some(logs_dir) = logs_dir else {
    fatal_error("Your config file does not have an EverQuest directory, not did you specify the Logs dir path with -L");
  };

  if let Err(e) = reactor::start(&logs_dir) {
    fatal_error(&e.to_string());
  }

  let app_state = AppState::init_from_config(config)?;
  ui::launch(app_state);
  Ok(())
}
