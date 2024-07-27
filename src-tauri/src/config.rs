use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use tracing::info;
use ts_rs::TS;

const CONFIG_FILE_NAME: &str = "LogQuest.toml";

#[derive(TS, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LogQuestConfig {
  pub everquest_directory: Option<String>,

  #[serde(skip)]
  #[ts(skip)]
  pub config_file_path: PathBuf,
}

impl LogQuestConfig {
  fn new_with_path(config_file_path: &Path) -> Self {
    LogQuestConfig {
      everquest_directory: None,
      config_file_path: config_file_path.to_owned(),
    }
  }

  fn load_from_file_path(path: &Path) -> anyhow::Result<Self> {
    let raw_config_file = fs::read_to_string(path)?;
    let mut config: LogQuestConfig = toml::from_str(&raw_config_file)?;
    config.config_file_path = path.to_owned();
    Ok(config)
  }

  pub fn save(&self) -> anyhow::Result<()> {
    self.write_to_file_path(&self.config_file_path)
  }

  fn write_to_file_path(&self, path: &Path) -> anyhow::Result<()> {
    let raw_toml = toml::to_string_pretty(&self)?;

    if let Some(parent) = path.parent() {
      if !parent.exists() {
        info!("Creating directory {}", parent.display());
        fs::create_dir_all(&parent)?;
      }
    }
    let mut file = fs::File::create(path)?; // overwrites file if it already exists
    file.write_all(raw_toml.as_bytes())?;
    Ok(())
  }
}

pub fn load_or_create_app_config_from_dir(config_dir: &PathBuf) -> anyhow::Result<LogQuestConfig> {
  let config_path = config_dir.join(&CONFIG_FILE_NAME);

  let config = if config_path.exists() {
    LogQuestConfig::load_from_file_path(&config_path)?
  } else {
    let config = LogQuestConfig::new_with_path(&config_path);
    config.save()?;
    config
  };
  Ok(config)
}

pub fn get_config_dir_with_optional_override(
  path_override: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
  let config_dir = match path_override {
    Some(overridden_dir) => overridden_dir,
    None => default_config_dir()?,
  };
  fs::create_dir_all(&config_dir)?;
  Ok(config_dir)
}

pub fn default_config_dir() -> anyhow::Result<PathBuf> {
  let mut cfg_dir = dirs::config_dir().ok_or_else(|| anyhow!("Could not determine the config directory. Please set the config directory manually with the --config-dir flag"))?;
  let app_name = env!("CARGO_PKG_NAME");
  cfg_dir.push(app_name);
  Ok(cfg_dir)
}
