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
use cli::cmd_with_optional_env_override;
use cli::{CLICommand, StartCommand, TTSCommand};
use common::fatal_if_err;
use state::config::LogQuestConfig;
use state::state_handle::StateHandle;
use state::state_tree::StateTree;
use tracing_subscriber::EnvFilter;
use triggers::TriggerLoadOrCreateError;

const DEFAULT_LOG_LEVEL: &str = "debug";

#[derive(thiserror::Error, Debug)]
enum AppStartError {
  #[error("Error processing config")]
  ConfigError(#[from] config::ConfigLoadOrCreateError),
  #[error("Failed processing Triggers")]
  TriggerError(#[from] TriggerLoadOrCreateError),
}

fn init_tracing() {
  let env_filter = EnvFilter::try_from_default_env();
  let env_filter = env_filter.unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL));
  tracing_subscriber::fmt().with_env_filter(env_filter).init()
}

fn main() {
  init_tracing();

  match cmd_with_optional_env_override() {
    CLICommand::Start(start_command) => fatal_if_err(start(start_command)),

    CLICommand::PrintAudioDevices => audio::print_audio_devices(), // returns `never`

    CLICommand::TTS(tts) => match tts {
      TTSCommand::Speak { message, voice } => fatal_if_err(tts::speak_once(message, voice)),
      TTSCommand::ListVoices => tts::print_voices(),
    },

    #[cfg(debug_assertions)]
    CLICommand::TypeScript => fatal_if_err(debug_only::generate_typescript()),

    #[cfg(debug_assertions)]
    CLICommand::Tail { file } => fatal_if_err(debug_only::tail(&file)),

    #[cfg(debug_assertions)]
    CLICommand::ConvertGINA { file, format, out } => debug_only::convert_gina(&file, format, out),
  };
}

fn start(
  StartCommand {
    config_dir,
    logs_dir,
    overlay_mode,
    overlay_dev_tools,
  }: StartCommand,
) -> Result<(), AppStartError> {
  print_banner();
  let config_dir = config::get_config_dir_with_optional_override(config_dir);
  let config = LogQuestConfig::load_or_create_in_dir(&config_dir, &logs_dir)?;
  let triggers = triggers::load_or_create_relative_to_config(&config)?; // TODO: Need to report JSON parse errors somewhere
  let state_tree = StateTree::new(config, triggers, overlay_mode, overlay_dev_tools);
  let state_handle = StateHandle::new(state_tree);
  ui::launch(state_handle);
  Ok(())
}

fn print_banner() {
  println!(
    "{}",
    r#"
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
             ┓     ┏┓
             ┃ ┏┓┏┓┃┃┓┏┏┓┏╋
             ┗┛┗┛┗┫┗┻┗┻┗ ┛┗
                  ┛
           the Deluxe Toolbox
         for EverQuest enjoyers
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
"#
  )
}
