// The next line prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(debug_assertions)]
mod debug_only;

mod audio;
mod cli;
mod commands;
mod common;
mod gina;
mod logs;
mod matchers;
mod reactor;
mod state;
mod triggers;
mod tts;
mod ui;

use crate::state::config;
use cli::{Commands, StartCommand, TTSCommand};
use common::fatal_error;
use state::config::LogQuestConfig;
use state::state_handle::StateHandle;
use state::state_tree::StateTree;
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

    Commands::PrintAudioDevices => print_audio_devices(),

    Commands::TTS(tts) => match tts {
      TTSCommand::Speak { message, voice } => tts::speak_once(message, voice),
      TTSCommand::ListVoices => tts::print_voices(),
    },

    #[cfg(debug_assertions)]
    Commands::TypeScript => debug_only::generate_typescript(),

    #[cfg(debug_assertions)]
    Commands::Tail { file } => debug_only::tail(&file),

    #[cfg(debug_assertions)]
    Commands::ConvertGINA { file, format } => debug_only::convert_gina(&file, format),
  };
  if let Err(e) = result {
    fatal_error(format!("{:?}", e));
  }
}

fn start(
  config_dir_override: Option<PathBuf>,
  logs_dir_override: Option<PathBuf>,
) -> anyhow::Result<()> {
  let config_dir = config::get_config_dir_with_optional_override(config_dir_override);
  let config = LogQuestConfig::load_or_create_in_dir(&config_dir, &logs_dir_override)?;
  let triggers = triggers::load_or_create_relative_to_config(&config)?; // TODO: Need to report JSON parse errors somewhere
  let state_tree = StateTree::init_from_configs(config, triggers)?;
  let state_handle = StateHandle::new(state_tree);

  ui::launch(state_handle);
  Ok(())
}

fn print_audio_devices() -> ! {
  let devices = audio::get_device_names();
  if devices.is_empty() {
    fatal_error("No audio devices found!");
  }
  println!("\nAudio devices detected:\n");
  for device_name in devices.iter() {
    println!(" - {device_name}");
  }
  println!("");
  std::process::exit(0);
}
