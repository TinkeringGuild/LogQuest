use crate::common::shutdown::critical_path;
use crate::common::{absolute_path_handling_tilde, fatal_error, format_integer};
use crate::triggers::TriggerRoot;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use tracing::error;
use tracing::{debug, info};
use ts_rs::TS;

const CONFIG_FILE_NAME: &str = "LogQuest.toml";
const TRIGGERS_FILE_NAME: &str = "Triggers.json";

#[derive(thiserror::Error, Debug)]
pub enum EverQuestDirectoryError {
  #[error("Directory does not exist")]
  DoesNotExist,
  #[error("Directory does not have a Logs sub-directory")]
  DoesNotHaveLogsDir,
  #[error("Directory is not a valid EverQuest installation")]
  NotValidEverQuestDir,
  #[error("Could not determine the absolute path of the directory")]
  CouldNotCanonicalize,
  #[error("The path appears corrupted with invalid characters")]
  CorruptedPath,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigLoadError {
  #[error("Encountered IO error loading the config")]
  IOError(#[from] std::io::Error),
  #[error("Encountered an error parsing the config file TOML")]
  TOMLDeserializationError(#[from] toml::de::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigLoadOrCreateError {
  #[error("Loading config failed!")]
  LoadError(#[from] ConfigLoadError),
  #[error("Creating config file failed!")]
  CreateError(#[from] ConfigSaveError),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigSaveError {
  #[error("Encountered IO error saving the config")]
  IOError(#[from] std::io::Error),
  #[error("Encountered an error serializing the config file TOML")]
  TOMLSerializationError(#[from] toml::ser::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TriggersSaveError {
  #[error("Encountered IO error saving the Triggers")]
  IOError(#[from] std::io::Error),
  #[error("Encountered an error serializing the Triggers JSON file")]
  JSONSerializationError(#[from] serde_json::error::Error),
}

#[derive(TS, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LogQuestConfig {
  everquest_directory: Option<String>,

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
  ) -> Result<LogQuestConfig, ConfigLoadOrCreateError> {
    let config_path = config_dir.join(&CONFIG_FILE_NAME);
    let config = if config_path.exists() {
      info!("Loading configuration from {}", config_dir.display());
      LogQuestConfig::load_from_file_path(&config_path, logs_dir_override)?
    } else {
      info!(
        "No config found. Creating fresh config in {}",
        config_dir.display()
      );
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
  fn load_from_file_path(
    path: &Path,
    logs_dir_override: &Option<PathBuf>,
  ) -> Result<Self, ConfigLoadError> {
    let raw_config_file = fs::read_to_string(path)?;
    let mut config: LogQuestConfig = toml::from_str(&raw_config_file)?;
    if let Some(eq_dir) = config.everquest_directory {
      config.everquest_directory = validate_eq_dir(&eq_dir).ok();
    }
    config.logs_dir_path = match (&config.everquest_directory, logs_dir_override) {
      (_, Some(overridden)) => {
        let Ok(logs_dir) = absolute_path_handling_tilde(overridden) else {
          fatal_error(&format!(
            "Could not determine absolute path of log dir override param: {}",
            overridden.display()
          ));
        };
        Some(logs_dir)
      }
      (Some(eq_dir), _) => default_logs_dir_from_eq_dir(&eq_dir).ok(),
      _ => None,
    };
    config.config_file_path = path.to_owned();
    Ok(config)
  }

  pub fn save_config(&self) -> Result<(), ConfigSaveError> {
    let pretty_toml = toml::to_string_pretty(&self)?;

    if let Some(parent) = self.config_file_path.parent() {
      if !parent.exists() {
        info!("Creating directory {}", parent.display());
        fs::create_dir_all(&parent)?;
      }
    }

    critical_path(|| {
      let mut file = fs::File::create(&self.config_file_path)?;
      file.write_all(pretty_toml.as_bytes())
    })?;

    Ok(())
  }

  pub fn save_triggers(&self, root: &TriggerRoot) -> Result<(), TriggersSaveError> {
    let triggers_file_path = self.triggers_file_path();
    let pretty_json = serde_json::to_string_pretty(&root)?;
    let json_bytes = pretty_json.into_bytes();
    let json_size = json_bytes.len();

    critical_path(|| {
      let mut file = fs::File::create(&triggers_file_path)?;
      file.write_all(&json_bytes)
    })?;

    debug!(
      "Wrote {} bytes to {TRIGGERS_FILE_NAME}",
      format_integer(json_size)
    );
    Ok(())
  }

  pub fn triggers_file_path(&self) -> PathBuf {
    self
      .config_file_path
      .parent()
      .expect("Could not determine parent directory of the config file!")
      .join(TRIGGERS_FILE_NAME)
  }

  /// This function should always be used to set the everquest_directory field because
  /// it validates the value and automatically sets the Logs dir from it (if one is
  /// not currently set via a CLI override). NOTE! This does not automatically notify
  /// `config_updated` on the `StateHandle`, so this should be called within a `StateHandle`
  /// mutex-value accessor higher-order-function.
  pub fn set_eq_dir(&mut self, path: &str) -> Result<String, EverQuestDirectoryError> {
    let eq_dir = validate_eq_dir(path)?;
    self.everquest_directory = Some(eq_dir.clone());
    if self.logs_dir_path.is_none() {
      self.logs_dir_path = Some(default_logs_dir_from_eq_dir(&eq_dir)?);
    }
    Ok(eq_dir)
  }

  /// LogQuest can be started before the reactor is able to begin; if this happens, it will
  /// poll this function anytime the config is updated.
  pub fn is_ready(&self) -> bool {
    self.everquest_directory.is_some() && self.logs_dir_path.is_some()
  }
}

/// By default, this uses the platform-specific conventional config directory as the parent
/// directory of the config dir.
///
/// On Linux/macOS, this should be `$XDG_CONFIG_HOME` or `~/.config`
/// On Windows, this should be `C:\Users\<Username>\AppData\Roaming`
///
/// Since the path can be overridden at the LQ CLI, this is a convenience method that takes
/// the `Option` directly and decides how to use it.
///
/// This function also ensures the directory and any parent directories are created. It will
/// panic if the directories could not be created, since this is essentially unrecoverable.
pub fn get_config_dir_with_optional_override(path_override: Option<PathBuf>) -> PathBuf {
  let config_dir = path_override.unwrap_or_else(default_config_dir);
  let Ok(config_dir) = absolute_path_handling_tilde(&config_dir) else {
    fatal_error(&format!(
      "Could not determine absolute path of config dir: {}",
      config_dir.display()
    ));
  };
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

pub fn validate_eq_dir(dir: &str) -> Result<String, EverQuestDirectoryError> {
  let path: PathBuf = dir.into();
  if !path.is_dir() {
    error!("The EQ directory given does not exist!");
    return Err(EverQuestDirectoryError::DoesNotExist);
  }
  if !path.join("eqclient.ini").exists() {
    error!("The EQ directory given does not appear to be a valid EverQuest installation: {dir}");
    return Err(EverQuestDirectoryError::NotValidEverQuestDir);
  }
  let Ok(path) = path.canonicalize() else {
    error!("Could not canonicalize the EQ directory path: {dir}");
    return Err(EverQuestDirectoryError::CouldNotCanonicalize);
  };
  let Some(path) = path.to_str() else {
    error!("The path to the EQ directory appears corrupted! Invalid UTF characters?");
    return Err(EverQuestDirectoryError::CorruptedPath);
  };
  Ok(path.to_owned())
}

fn default_logs_dir_from_eq_dir(eq_path: &str) -> Result<PathBuf, EverQuestDirectoryError> {
  let eq_path: PathBuf = eq_path.into();
  let logs_dir = eq_path.join("Logs");
  if !logs_dir.is_dir() {
    error!(
      "The EQ directory has no Logs sub-directory! Missing path: {}",
      logs_dir.display()
    );
    return Err(EverQuestDirectoryError::DoesNotHaveLogsDir);
  }
  Ok(logs_dir)
}
