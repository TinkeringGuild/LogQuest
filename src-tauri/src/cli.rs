use clap::{command, Parser, Subcommand};
use std::path::PathBuf;

pub(crate) fn cmd() -> Commands {
  if let Ok(cli) = CLI::try_parse() {
    return cli.command.unwrap_or_else(default_command); // the ...or_else case happens when called with zero args
  }

  // To support passing "start" subcommand params without specifying "start" as the first CLI arg, the
  // logic needs to try parsing the CLI with "start" prepended to the params list.
  let mut args: Vec<String> = std::env::args().collect();
  args.insert(1, "start".into()); // the 0th index is the program path
  if let Ok(cli) = CLI::try_parse_from(args.iter()) {
    return cli
      .command
      .expect(r#""start" was inserted into args! The command should be "start"!"#);
  }

  // Since neither parse attempt worked, call CLI::parse() again and it let exit the program for us
  // and report the help text normally.
  CLI::parse();
  unreachable!();
}

#[derive(Parser)]
#[command(
  name = "LogQuest",
  author = "Tinkering Guild",
  about = "Log parser, overlay UI, notification system, and all-around Deluxe Toolbox for EverQuest enjoyers"
)]
pub(crate) struct CLI {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
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
    file: PathBuf,

    #[arg(
      value_enum, long, short,
      default_value_t=ConvertGinaFormat::JSON,
      help = "Specify the format of the output"
    )]
    format: ConvertGinaFormat,

    #[arg(long, help = "Specify a file to write the output to")]
    out: Option<PathBuf>,
  },
}

#[cfg(debug_assertions)]
#[derive(clap::ValueEnum, Debug, Clone)]
pub(crate) enum ConvertGinaFormat {
  JSON,
  Internal,
  GinaInternal,
  GinaJSON,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct StartCommand {
  /// Override the path to the LogQuest configuration directory
  #[arg(long = "config-dir", short = 'C')]
  pub(crate) config_dir: Option<PathBuf>,

  /// Override the path to EverQuest's logs
  #[arg(long = "logs-dir", short = 'L')]
  pub(crate) logs_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TTSCommand {
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

fn default_command() -> Commands {
  Commands::Start(StartCommand::parse_from(std::env::args()))
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  CLI::command().debug_assert();
}
