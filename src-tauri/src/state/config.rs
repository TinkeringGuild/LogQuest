use crate::common::shutdown::critical_path;
use crate::common::{absolute_path_handling_tilde, fatal_error, format_integer, UUID, UUID_LEN};
use crate::triggers::trigger_index::{
  DataMutationError, TriggerGroupDescendant, TriggerIndex, TriggerTag,
};
use crate::triggers::{Trigger, TriggerGroup};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tracing::error;
use tracing::{debug, info};

const CONFIG_FILE_NAME: &str = "LogQuest.toml";
const DATA_DIR_NAME: &str = "Data";
const TRIGGERS_DIR_NAME: &str = "Triggers";
const TRIGGER_GROUPS_DIR_NAME: &str = "Groups";
const TRIGGER_TAGS_DIR_NAME: &str = "TriggerTags";
const TOP_LEVEL_FILE_NAME: &str = "tree.json";

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
  IOError(#[from] io::Error),
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
  IOError(#[from] io::Error),
  #[error("Encountered an error serializing the config file TOML")]
  TOMLSerializationError(#[from] toml::ser::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TriggersSaveError {
  #[error(transparent)]
  IOError(#[from] io::Error),
  #[error(transparent)]
  JSONSerializationError(#[from] serde_json::error::Error),
  #[error(transparent)]
  MutationError(#[from] DataMutationError),
}

#[derive(thiserror::Error, Debug)]
pub enum TriggerLoadError {
  #[error(transparent)]
  IOError(#[from] io::Error),
  #[error(transparent)]
  JSONDeserializationError(#[from] serde_json::error::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ts_rs::TS)]
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
    let serialized_config = fs::read_to_string(path)?;
    let mut config: LogQuestConfig = toml::from_str(&serialized_config)?;
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
    ensure_dir_exists(self.config_dir_path());
    critical_path(|| {
      let mut file = fs::File::create(&self.config_file_path)?;
      file
        .write_all(pretty_toml.as_bytes())
        .and_then(|_| file.flush())
    })?;
    Ok(())
  }

  pub fn save_trigger_index(&self, index: &TriggerIndex) -> Result<(), TriggersSaveError> {
    self.remove_id_files_not_in_iter(&self.triggers_dir_path(), index.triggers.keys())?;
    for trigger in index.triggers.values() {
      self.save_trigger(&trigger)?;
    }

    self.remove_id_files_not_in_iter(&self.trigger_groups_dir_path(), index.groups.keys())?;
    for group in index.groups.values() {
      self.save_trigger_group(&group)?;
    }

    self.remove_id_files_not_in_iter(&self.trigger_tags_dir_path(), index.trigger_tags.keys())?;
    for trigger_tag in index.trigger_tags.values() {
      self.save_trigger_tag(trigger_tag)?;
    }

    self.save_top_level(&index.top_level)?;

    Ok(())
  }

  fn ids_in_dir(&self, dir: &Path) -> io::Result<Vec<UUID>> {
    const EXPECTED_FILENAME_LEN: usize = UUID_LEN + ".json".len();
    let dir_entries: Vec<fs::DirEntry> = dir.read_dir()?.collect::<Result<Vec<_>, _>>()?;
    let uuids = dir_entries
      .iter()
      .map(|entry| entry.file_name().to_string_lossy().into_owned())
      .filter(|name| name.len() == EXPECTED_FILENAME_LEN && name.ends_with(".json"))
      .map(|file_name| UUID::from_str_unchecked(&file_name[..UUID_LEN]))
      .collect();
    Ok(uuids)
  }

  fn remove_id_files_not_in_iter<'a, I>(&'a self, dir: &Path, excluded: I) -> io::Result<()>
  where
    I: Iterator<Item = &'a UUID>,
  {
    let persisted_ids = self.ids_in_dir(dir)?;
    let persisted_ids_set: HashSet<&UUID> = HashSet::from_iter(persisted_ids.iter());
    let exclusion_set: HashSet<&UUID> = HashSet::from_iter(excluded);

    for id in persisted_ids_set.difference(&exclusion_set) {
      let file_path = id_file_in(dir, id);
      debug!("Removing stale data file: {}", file_path.display());
      fs::remove_file(file_path)?
    }

    Ok(())
  }

  pub fn save_trigger(&self, trigger: &Trigger) -> Result<(), TriggersSaveError> {
    let trigger_file_path = self.trigger_file_path(&trigger.id);
    self.write_json(trigger, &trigger_file_path)
  }

  pub fn save_trigger_group(&self, group: &TriggerGroup) -> Result<(), TriggersSaveError> {
    let group_file_path = self.trigger_group_file_path(&group.id);
    self.write_json(group, &group_file_path)
  }

  pub fn save_trigger_tag(&self, tag: &TriggerTag) -> Result<(), TriggersSaveError> {
    let tag_file_path = self.trigger_tag_file_path(&tag.id);
    self.write_json(tag, &tag_file_path)
  }

  pub fn save_top_level(
    &self,
    top_level: &Vec<TriggerGroupDescendant>,
  ) -> Result<(), TriggersSaveError> {
    self.write_json(top_level, &self.top_level_file_path())
  }

  fn write_json<S>(&self, value: S, path: &Path) -> Result<(), TriggersSaveError>
  where
    S: Serialize,
  {
    let pretty_json = serde_json::to_string_pretty(&value)?;
    let json_bytes = pretty_json.into_bytes();
    let json_size = json_bytes.len();

    let mut file = fs::File::create(&path)?;
    file.write_all(&json_bytes)?;
    file.flush()?;

    debug!(
      "Wrote {} bytes to {}",
      format_integer(json_size),
      path.display()
    );
    Ok(())
  }

  /// NOTE: This does NOT automatically call security_check on the deserialized Triggers
  pub fn load_all_triggers(&self) -> Result<Vec<Trigger>, TriggerLoadError> {
    let triggers: Result<Vec<Trigger>, TriggerLoadError> = self
      .ids_in_dir(&self.triggers_dir_path())?
      .into_iter()
      .map(|id| self.load_trigger_file(&id))
      .collect();
    Ok(triggers?)
  }

  pub fn load_all_trigger_groups(&self) -> Result<Vec<TriggerGroup>, TriggerLoadError> {
    let groups: Result<Vec<TriggerGroup>, TriggerLoadError> = self
      .ids_in_dir(&self.trigger_groups_dir_path())?
      .into_iter()
      .map(|id| self.load_trigger_group_file(&id))
      .collect();
    Ok(groups?)
  }

  pub fn load_all_trigger_tags(&self) -> Result<Vec<TriggerTag>, TriggerLoadError> {
    let tags: Result<Vec<TriggerTag>, TriggerLoadError> = self
      .ids_in_dir(&self.trigger_tags_dir_path())?
      .into_iter()
      .map(|id| self.load_trigger_tag_file(&id))
      .collect();
    Ok(tags?)
  }

  pub fn load_trigger_file(&self, id: &UUID) -> Result<Trigger, TriggerLoadError> {
    let file = fs::File::open(self.trigger_file_path(id))?;
    serde_json::from_reader(BufReader::new(file)).map_err(|e| e.into())
  }

  pub fn load_trigger_group_file(&self, id: &UUID) -> Result<TriggerGroup, TriggerLoadError> {
    let file = fs::File::open(self.trigger_group_file_path(id))?;
    serde_json::from_reader(BufReader::new(file)).map_err(|e| e.into())
  }

  pub fn load_trigger_tag_file(&self, id: &UUID) -> Result<TriggerTag, TriggerLoadError> {
    let file = fs::File::open(self.trigger_tag_file_path(id))?;
    serde_json::from_reader(BufReader::new(file)).map_err(|e| e.into())
  }

  pub fn load_top_level_file(
    &self,
  ) -> Result<Option<Vec<TriggerGroupDescendant>>, TriggerLoadError> {
    let file_path = self.top_level_file_path();
    if !file_path.exists() {
      return Ok(None);
    }
    let file = fs::File::open(file_path)?;
    let top_level: Vec<TriggerGroupDescendant> = serde_json::from_reader(BufReader::new(file))?;
    Ok(Some(top_level))
  }

  pub fn delete_trigger_tag_file(&self, trigger_tag_id: &UUID) -> io::Result<()> {
    let path = id_file_in(self.trigger_tags_dir_path(), trigger_tag_id);
    if path.is_file() {
      fs::remove_file(path)?;
    }
    Ok(())
  }

  fn trigger_file_path<S>(&self, id: S) -> PathBuf
  where
    S: AsRef<str>,
  {
    id_file_in(self.triggers_dir_path(), id)
  }

  fn trigger_group_file_path(&self, id: &UUID) -> PathBuf {
    id_file_in(self.trigger_groups_dir_path(), id)
  }

  fn trigger_tag_file_path(&self, id: &UUID) -> PathBuf {
    id_file_in(self.trigger_tags_dir_path(), id)
  }

  fn top_level_file_path(&self) -> PathBuf {
    self.data_dir_path().join(TOP_LEVEL_FILE_NAME)
  }

  fn data_dir_path(&self) -> PathBuf {
    self.config_dir_path().join(DATA_DIR_NAME)
  }

  fn triggers_dir_path(&self) -> PathBuf {
    ensure_dir_exists(self.data_dir_path().join(TRIGGERS_DIR_NAME))
  }

  fn trigger_groups_dir_path(&self) -> PathBuf {
    ensure_dir_exists(self.data_dir_path().join(TRIGGER_GROUPS_DIR_NAME))
  }

  fn trigger_tags_dir_path(&self) -> PathBuf {
    ensure_dir_exists(self.data_dir_path().join(TRIGGER_TAGS_DIR_NAME))
  }

  fn config_dir_path(&self) -> PathBuf {
    self
      .config_file_path
      .parent()
      .expect("Could not determine parent directory of the config file!")
      .to_owned()
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
  ensure_dir_exists(config_dir)
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

fn ensure_dir_exists<P>(path: P) -> P
where
  P: AsRef<Path>,
{
  let path_ref = path.as_ref();
  if path_ref.is_file() {
    fatal_error(format!(
      "Could not create directory {}. It already exists as a file!",
      path_ref.display()
    ));
  } else if !path_ref.is_dir() {
    fs::create_dir_all(&path_ref).expect(&format!(
      "Could not create directory: {}",
      path_ref.display()
    ));
  }
  path
}

fn id_file_in<P, U>(dir: P, id: U) -> PathBuf
where
  P: AsRef<Path>,
  U: AsRef<str>,
{
  dir.as_ref().join(format!("{}.json", id.as_ref()))
}
