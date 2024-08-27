use crate::common::fatal_error;
use crate::state::overlay::OverlayMode;
use clap::{command, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

const LQ_CMD_OVERRIDE_ENV_VAR: &str = "LQ";

/// If LogQuest is ran with a LQ environment variable, it will prioritize
/// extracting its CLI params from that over those given via `std::env::args`.
/// This is particularly useful when used with `npm run tauri dev` because Tauri
/// doesn't allow any way (AFAIK) to pass through CLI params to the Rust process
/// when running the development server. NOTE: If you pass in LQ, you
/// must give args that resolve to a command; for example, commands that end with
/// "--help" are not considered commands by `clap`.
pub fn cmd_with_optional_env_override() -> CLICommand {
  if let Ok(params_override) = env::var(LQ_CMD_OVERRIDE_ENV_VAR) {
    let params: Vec<String> = env::args()
      .take(1)
      .chain(
        params_override
          .split_whitespace()
          .into_iter()
          .map(String::from),
      )
      .collect();
    match CLI::try_parse_from(params.clone()) {
      Ok(CLI {
        command: Some(command),
      }) => return command,
      _ => fatal_error(format!(
        "Could not parse {LQ_CMD_OVERRIDE_ENV_VAR} env var: `{params:?}`"
      )),
    }
  }
  cmd()
}

/// This function parses the CLI params with extra logic to accommodate a shorthand way of starting
/// LogQuest where "start" is implied if no command is given, and all further CLI params are treated
/// as params to StartCommand. If an explicit subcommand is given, this defers to the normal logic.
pub fn cmd() -> CLICommand {
  if let Ok(cli) = CLI::try_parse() {
    return cli.command.unwrap_or_else(default_command); // the ...or_else case happens when called with zero args
  }

  // To support passing "start" subcommand params without specifying "start" as the first CLI arg, the
  // logic needs to try parsing the CLI with "start" prepended to the params list.
  let mut args: Vec<String> = env::args().collect();
  args.insert(1, "start".into()); // the 0th index is the program path
  if let Ok(cli) = CLI::try_parse_from(args.iter()) {
    return cli
      .command
      .expect(r#""start" was inserted into args! The command should be "start"!"#);
  }

  // Since neither parse attempt worked, call CLI::parse() again and let it exit the program for us
  // and report the help text normally.
  CLI::parse(); // will always exit the process here with help text
  unreachable!();
}

#[derive(Parser)]
#[command(
  name = "LogQuest",
  author = "Tinkering Guild",
  about = "Log parser, overlay UI, notification system, and all-around Deluxe Toolbox for EverQuest enjoyers"
)]
pub struct CLI {
  #[command(subcommand)]
  pub command: Option<CLICommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CLICommand {
  /// Start LogQuest normally
  Start(StartCommand),

  /// Prints out all detected audio devices
  PrintAudioDevices,

  /// Utilities for tinkering with the LogQuest text-to-speech engine
  #[command(subcommand)]
  TTS(TTSCommand),

  /// (DEBUG BUILDS ONLY) Generate and save TypeScript type definition files
  #[cfg(debug_assertions)]
  #[command(name = "typescript")]
  TypeScript,

  /// (DEBUG BUILDS ONLY) Specify a file path to watch filesystem create/modify/delete events
  #[cfg(debug_assertions)]
  Tail { file: PathBuf },

  /// (DEBUG BUILDS ONLY) Utiliy for inspecting or converting a GINA .xml or .gtp file
  #[cfg(debug_assertions)]
  ConvertGINA {
    /// The path to the GINA .gtp or .xml file
    file: PathBuf,

    /// Specify the format of the output
    #[arg(
      value_enum, long, short,
      default_value_t=ConvertGinaFormat::JSON
    )]
    format: ConvertGinaFormat,

    /// Specify a file to write the output into
    #[arg(long)]
    out: Option<PathBuf>,
  },
}

#[cfg(debug_assertions)]
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ConvertGinaFormat {
  JSON,
  Internal,
  GinaInternal,
  GinaJSON,
}

#[derive(Parser, Debug, Clone)]
pub struct StartCommand {
  /// Override the path to the LogQuest configuration directory
  #[arg(long = "config-dir", short = 'C')]
  pub config_dir_override: Option<PathBuf>,

  /// Override the path to EverQuest's logs
  #[arg(long = "logs-dir", short = 'L')]
  pub logs_dir_override: Option<PathBuf>,

  /// Specify how the overlay should be shown
  #[arg(long="overlay", value_enum, default_value_t=OverlayMode::Default)]
  pub overlay_mode: OverlayMode,

  /// If given, this will automatically open the dev tools for the overlay window
  #[arg(long)]
  pub overlay_dev_tools: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TTSCommand {
  /// Speak a message with text-to-speech. You can specify a specific voice.
  Speak {
    /// Provide a quoted string to speak
    message: String,
    /// Specify a specific voice. Use "tts list-voices" subcommand to see available options.
    #[arg(long)]
    voice: Option<String>,
  },
  /// Prints out the available voices for your system's text-to-speech engine in CSV format
  ListVoices,
}

fn default_command() -> CLICommand {
  CLICommand::Start(StartCommand::parse_from(env::args()))
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  CLI::command().debug_assert();
}
