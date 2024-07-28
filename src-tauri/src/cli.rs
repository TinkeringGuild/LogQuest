use clap::{command, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub(crate) fn cmd() -> Commands {
  let cli = CLI::parse();
  cli.command.unwrap_or_else(default_command)
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

#[derive(Debug, Subcommand, Clone)]
pub(crate) enum Commands {
  /// Start LogQuest normally
  Start(StartCommand),

  /// (DEBUG BUILDS ONLY) Generate and save TypeScript type definition files
  #[cfg(debug_assertions)]
  TS,

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
  },
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct StartCommand {
  #[arg(long = "config-dir", short = 'C')]
  /// Override the path to the LogQuest configuration directory
  pub(crate) config_dir: Option<PathBuf>,
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  CLI::command().debug_assert();
}

fn default_command() -> Commands {
  Commands::Start(StartCommand::parse_from(std::env::args()))
}

#[cfg(debug_assertions)]
#[derive(ValueEnum, Debug, Clone)]
pub(crate) enum ConvertGinaFormat {
  JSON,
  Internal,
  GinaInternal,
  GinaJSON,
}
