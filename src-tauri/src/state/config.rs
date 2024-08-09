use crate::common::fatal_error;
use crate::triggers::TriggerRoot;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use ts_rs::TS;

const CONFIG_FILE_NAME: &str = "LogQuest.toml";
const TRIGGERS_FILE_NAME: &str = "Triggers.json";

#[derive(TS, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LogQuestConfig {
  pub everquest_directory: Option<String>,

  #[serde(skip)]
  #[ts(skip)]
  pub config_file_path: PathBuf,

  /// By default the logs dir can be inferred from the everquest_directory
  /// but the Logs dir can be overridden at the CLI, so it should always
  /// be accessed here for the correct value.
  #[serde(skip)]
  #[ts(skip)]
  pub logs_dir_path: Option<PathBuf>,
}
impl LogQuestConfig {
  /// Given the config directory, load the LogQuestConfig from it or
  /// create a new one on-disk and return it.
  pub fn load_or_create_in_dir(
    config_dir: &PathBuf,
    logs_dir_override: &Option<PathBuf>,
  ) -> anyhow::Result<LogQuestConfig> {
    let config_path = config_dir.join(&CONFIG_FILE_NAME);
    let config = if config_path.exists() {
      LogQuestConfig::load_from_file_path(&config_path, logs_dir_override)?
    } else {
      let config = LogQuestConfig::new_with_config_file_path(&config_path);
      config.save_config()?;
      config
    };
    Ok(config)
  }

  /// Initializes a new LogQuestConfig for when none could be loaded from the filesystem
  fn new_with_config_file_path(config_file_path: &Path) -> Self {
    if !config_file_path.exists() {
      fatal_error(format!(
        "Config file path does not exist! Path: {}",
        config_file_path.display()
      ));
    }
    LogQuestConfig {
      config_file_path: config_file_path.to_owned(),
      logs_dir_path: None,
      everquest_directory: None,
    }
  }

  /// Loads a LogQuestConfig from the given path.
  fn load_from_file_path(path: &Path, logs_dir_override: &Option<PathBuf>) -> anyhow::Result<Self> {
    let raw_config_file = fs::read_to_string(path)?;
    let mut config: LogQuestConfig = toml::from_str(&raw_config_file)?;
    config.config_file_path = path.to_owned();
    config.logs_dir_path = match (config.everquest_directory.as_deref(), logs_dir_override) {
      (_, Some(logs_dir)) => Some(logs_dir.to_owned()),
      (Some(eq_dir), _) => Some(default_logs_dir_from_eq_dir(&PathBuf::from(eq_dir))),
      _ => None,
    };
    Ok(config)
  }

  pub fn save_config(&self) -> anyhow::Result<()> {
    let pretty_toml = toml::to_string_pretty(&self)?;

    if let Some(parent) = self.config_file_path.parent() {
      if !parent.exists() {
        info!("Creating directory {}", parent.display());
        fs::create_dir_all(&parent)?;
      }
    }

    let mut file = fs::File::create(&self.config_file_path)?;
    file.write_all(pretty_toml.as_bytes())?;

    Ok(())
  }

  pub fn save_triggers(&self, root: &TriggerRoot) -> Result<(), io::Error> {
    let triggers_file_path = self.triggers_file_path();
    let json_bytes = serde_json::to_string_pretty(&root).and_then(|s| Ok(s.into_bytes()))?;
    let json_size = json_bytes.len();

    let mut file = fs::File::create(&triggers_file_path)?;
    file.write_all(&json_bytes)?;

    debug!("Wrote {json_size} bytes to {TRIGGERS_FILE_NAME}");
    Ok(())
  }

  pub fn triggers_file_path(&self) -> PathBuf {
    self
      .config_file_path
      .parent()
      .expect("Could not determine parent directory of the config file!")
      .join(TRIGGERS_FILE_NAME)
  }
}

/// By default, this uses the platform-specific conventional config directory as the parent
/// directory of the config dir.
///
/// On Linux/macOS, this should be `$XDG_CONFIG_HOME` or `~/.config`
/// On Windows, this should be `C:\Users\<Username>\AppData\Roaming`
///
/// Since the path can be overridden at the LQ CLI, this is a convenience method that takes
/// the `Option` directly and decides whether how to use it.
///
/// This function also ensures the directory and any parent directories are created.
pub fn get_config_dir_with_optional_override(path_override: Option<PathBuf>) -> PathBuf {
  let config_dir = path_override.unwrap_or_else(default_config_dir);
  if let Err(e) = fs::create_dir_all(&config_dir) {
    fatal_error(format!(
      "Creating config dir failed: {}  [ ERROR: {e:?} ]",
      config_dir.display()
    ));
  }
  config_dir
}

pub fn default_config_dir() -> PathBuf {
  let Some(cfg_dir) = dirs::config_dir() else {
    fatal_error("Could not determine the config directory. Please set the config directory manually with the --config-dir flag");
  };
  let app_name = env!("CARGO_PKG_NAME");
  cfg_dir.join(app_name)
}

fn default_logs_dir_from_eq_dir(eq_path: &Path) -> PathBuf {
  eq_path.join("Logs")
}
