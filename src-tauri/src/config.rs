use anyhow::bail;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "LogQuest.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogQuestConfig {
    pub everquest_directory: Option<String>,
}

impl LogQuestConfig {
    fn load_from_file_path(path: PathBuf) -> anyhow::Result<Self> {
        let raw_config_file = fs::read_to_string(path)?;
        let config = toml::from_str(&raw_config_file)?;
        Ok(config)
    }

    fn write_to_file_path(&self, path: PathBuf) -> anyhow::Result<()> {
        let raw_toml = toml::to_string_pretty(&self)?;
        let mut file = if path.exists() {
            File::open(path)?
        } else {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    println!("Creating directory {}", parent.display());
                    fs::create_dir_all(&parent)?;
                }
            }
            println!("Creating file {}", path.display());
            File::create(path)?
        };
        file.write_all(raw_toml.as_bytes())?;
        Ok(())
    }
}

impl Default for LogQuestConfig {
    fn default() -> Self {
        LogQuestConfig {
            everquest_directory: None,
        }
    }
}

pub fn load_app_config(path_override: Option<PathBuf>) -> anyhow::Result<LogQuestConfig> {
    let mut config_path = match path_override {
        Some(path) => path,
        None => get_config_dir()?,
    };

    config_path.push(&CONFIG_FILE_NAME);

    if config_path.exists() {
        LogQuestConfig::load_from_file_path(config_path)
    } else {
        let config = LogQuestConfig::default();
        config.write_to_file_path(config_path)?;
        Ok(config)
    }
}

fn get_config_dir() -> anyhow::Result<PathBuf> {
    let Some(mut config_dir) = config_dir() else {
        bail!("Could not determine the config directory");
    };

    let app_name = env!("CARGO_PKG_NAME");
    config_dir.push(app_name);

    fs::create_dir_all(&config_dir)?;

    Ok(config_dir)
}
