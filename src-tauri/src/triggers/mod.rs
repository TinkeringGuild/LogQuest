pub mod command_template;
pub mod effects;
pub mod template_string;
pub mod timers;

use crate::{
  common::{timestamp::Timestamp, LogQuestVersion, LOG_QUEST_VERSION, UUID},
  gina::importer::GINAImport,
  matchers,
  state::config::{LogQuestConfig, TriggersSaveError},
};
use effects::EffectWithID;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use template_string::TemplateString;
use tracing::{debug, error};
use ts_rs::TS;

#[derive(thiserror::Error, Debug)]
pub enum TriggerLoadOrCreateError {
  #[error("Encountered IO error loading the Triggers")]
  IOError(#[from] std::io::Error),
  #[error("Encountered error deserializing the Triggers file")]
  DeserializationError(#[from] serde_json::Error),
  #[error("Failed to save the Triggers JSON file")]
  SaveError(#[from] TriggersSaveError),
}

#[derive(TS, Clone, Serialize, Deserialize)]
pub struct TriggerRoot {
  log_quest_version: LogQuestVersion,
  groups: Vec<TriggerGroup>,
}

impl TriggerRoot {
  pub fn new(groups: Vec<TriggerGroup>) -> Self {
    Self {
      log_quest_version: LOG_QUEST_VERSION.clone(),
      groups,
    }
  }

  pub fn find_mut_trigger_by_id(&mut self, id: &UUID) -> Option<&mut Trigger> {
    for group in self.groups.iter_mut() {
      if let Some(trigger) = group.find_mut_trigger_by_id(id) {
        return Some(trigger);
      }
    }
    error!("Attempted to find Trigger but it did no exist! [ ID = {id} ]");
    None
  }

  pub fn ingest_gina_import(&mut self, mut import: GINAImport) {
    self.groups.append(&mut import.converted)
  }

  pub fn iter(&self) -> std::slice::Iter<TriggerGroup> {
    self.groups.iter()
  }

  pub fn trigger_count(&self) -> usize {
    fn descend_descendants(descendants: &Vec<TriggerGroupDescendant>) -> usize {
      descendants.iter().fold(0, |sum, tgd| match tgd {
        TriggerGroupDescendant::T(_) => sum + 1,
        TriggerGroupDescendant::TG(tg) => sum + descend_descendants(&tg.children),
      })
    }
    self
      .groups
      .iter()
      .fold(0, |sum, tg| sum + descend_descendants(&tg.children))
  }

  pub fn security_check(self) -> Self {
    let groups = self
      .groups
      .into_iter()
      .map(|g| g.security_check())
      .collect();
    Self { groups, ..self }
  }
}

#[derive(TS, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TriggerGroup {
  pub id: UUID,
  pub name: String,
  pub comment: Option<String>,
  pub children: Vec<TriggerGroupDescendant>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
}

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerGroupDescendant {
  T(Trigger),
  TG(TriggerGroup),
}

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trigger {
  pub id: UUID,
  pub name: String,
  pub comment: Option<String>,
  pub enabled: bool,
  pub filter: matchers::Filter,
  pub effects: Vec<EffectWithID>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp, // tags: Vec<Tag>
}

impl From<TriggerGroup> for TriggerGroupDescendant {
  fn from(value: TriggerGroup) -> Self {
    TriggerGroupDescendant::TG(value)
  }
}

impl From<Trigger> for TriggerGroupDescendant {
  fn from(value: Trigger) -> Self {
    TriggerGroupDescendant::T(value)
  }
}

impl TriggerGroup {
  fn find_mut_trigger_by_id(&mut self, id: &UUID) -> Option<&mut Trigger> {
    for descendant in self.children.iter_mut() {
      match descendant {
        TriggerGroupDescendant::T(trigger) => {
          if trigger.id == *id {
            return Some(trigger);
          }
        }
        TriggerGroupDescendant::TG(group) => {
          if let Some(trigger) = group.find_mut_trigger_by_id(id) {
            return Some(trigger);
          }
        }
      }
    }
    None
  }

  fn security_check(self) -> Self {
    let children: Vec<TriggerGroupDescendant> = self
      .children
      .into_iter()
      .map(|tgd| tgd.security_check())
      .collect();
    Self { children, ..self }
  }
}

impl TriggerGroupDescendant {
  fn security_check(self) -> Self {
    match self {
      Self::T(trigger) => Self::T(trigger.security_check()),
      Self::TG(group) => Self::TG(group.security_check()),
    }
  }
}

impl Trigger {
  fn security_check(self) -> Self {
    let effects: Vec<EffectWithID> = self
      .effects
      .into_iter()
      .map(|e| e.security_check())
      .collect();
    Self { effects, ..self }
  }
}

pub fn load_or_create_relative_to_config(
  config: &LogQuestConfig,
) -> Result<TriggerRoot, TriggerLoadOrCreateError> {
  let triggers_file_path = config.triggers_file_path();
  if triggers_file_path.is_file() {
    debug!(
      "Triggers file exists. Loading {}",
      triggers_file_path.display()
    );
    let reader = BufReader::new(File::open(triggers_file_path)?);
    let root: TriggerRoot = serde_json::from_reader(reader)?;
    let root = root.security_check();
    Ok(root)
  } else {
    let root = default_triggers().security_check();
    config.save_triggers(&root)?;
    Ok(root)
  }
}

#[cfg(not(debug_assertions))]
pub fn default_triggers() -> TriggerRoot {
  TriggerRoot::new(vec![])
}

#[cfg(debug_assertions)]
pub fn default_triggers() -> TriggerRoot {
  TriggerRoot::new(vec![crate::debug_only::test_trigger_group()])
}

#[cfg(test)]
mod test {
  use super::{effects::Effect, EffectWithID, Trigger, TriggerGroup};
  use crate::{
    common::{timestamp::Timestamp, UUID},
    matchers::Matcher,
  };

  #[test]
  fn test_serde() {
    let tg_before = simple_sample();
    let raw_json =
      serde_json::to_string_pretty(&tg_before).expect("Could not convert TriggerGroup to JSON");
    let tg_after: TriggerGroup =
      serde_json::from_str(&raw_json).expect("Could not parse TriggerGroup JSON!");
    assert_eq!(tg_before, tg_after);
  }

  fn simple_sample() -> TriggerGroup {
    let now = Timestamp::now();
    let trigger = Trigger {
      id: UUID::new(),
      name: "Simple Sample Trigger 1".into(),
      enabled: true,
      comment: None,
      created_at: now.clone(),
      updated_at: now.clone(),
      filter: vec![Matcher::gina("^{S1} hits {S2}").unwrap()].into(),
      effects: vec![EffectWithID::new(Effect::Sequence(vec![
        Effect::Speak {
          tmpl: "This is only a test.".into(),
          interrupt: false,
        },
        Effect::PlayAudioFile(Some("/dev/null".into())),
      ]))],
    };
    TriggerGroup {
      id: UUID::new(),
      name: "Simple Sample Trigger Group".into(),
      comment: Some("I am a comment".into()),
      created_at: now.clone(),
      updated_at: now,
      children: vec![trigger.into()],
    }
  }
}
