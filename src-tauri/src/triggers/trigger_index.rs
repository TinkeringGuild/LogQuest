use super::{effects::Effect, template_string::TemplateString, Trigger, TriggerGroup};
use crate::common::{LogQuestVersion, UUID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cmp::min, collections::HashSet};
use tracing::error;

fn is_compatible_triggers_import_version(_version: &LogQuestVersion) -> bool {
  true // TODO: This will need to be maintained over time
}

pub type MutationResult = Result<Vec<DataDelta>, DataMutationError>;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct TriggerTag {
  pub id: UUID,
  name: String,
  triggers: HashSet<UUID>,
}

#[derive(thiserror::Error, Debug)]
#[error("Triggers file version is incompatible with this version of LogQuest. Found: {0:?}")]
struct OutdatedTriggersFile(LogQuestVersion);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum TriggerGroupDescendant {
  T(UUID),
  G(UUID),
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum Mutation {
  // TODO: Maybe have an SetTriggerField(TriggerFieldValue) mutation?
  SetTriggerName {
    trigger_id: UUID,
    new_name: String,
  },
  CreateTrigger {
    trigger: Trigger,
    parent_position: usize,
  },
  SaveTrigger(Trigger),
  CreateTriggerGroup {
    trigger_group: TriggerGroup,
    parent_position: usize,
  },
  CreateTriggerTag(String),
  DeleteTriggerTag(UUID),
  TagTrigger {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  UntagTrigger {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  EffectTemplateChanged {
    trigger_id: UUID,
    effect_id: UUID,
    tmpl: TemplateString,
  },
  EffectSpeakInterrupt {
    trigger_id: UUID,
    effect_id: UUID,
    interrupt: bool,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum DataDelta {
  TriggerSaved(Trigger),
  TriggerGroupCreated(TriggerGroup),
  TriggerGroupChildrenChanged {
    trigger_group_id: UUID,
    children: Vec<TriggerGroupDescendant>,
  },
  TopLevelChanged(Vec<TriggerGroupDescendant>),
  TriggerTagged {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  TriggerUntagged {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  TriggerTagCreated(TriggerTag),
  TriggerTagDeleted(UUID),
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize, ts_rs::TS)]
pub enum DataMutationError {
  #[error("Tried mutating a non-existent Trigger! ID: {0}")]
  TriggerNotFound(UUID),

  #[error("Tried mutating a non-existent TriggerGroup! ID: {0}")]
  TriggerGroupNotFound(UUID),

  #[error("Tried mutating a non-existent Tag! ID: {0}")]
  TriggerTagNotFound(UUID),

  #[error("Tried mutating a non-existent Effect! ID: {0}")]
  EffectNotFound(UUID),

  #[error("Tried performing an incorrect Effect mutation!")]
  IncorrectEffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct TriggerIndex {
  pub triggers: HashMap<UUID, Trigger>,
  pub groups: HashMap<UUID, TriggerGroup>,
  pub top_level: Vec<TriggerGroupDescendant>,
  pub trigger_tags: HashMap<UUID, TriggerTag>,
}

impl TriggerIndex {
  pub fn new() -> Self {
    Self {
      triggers: HashMap::new(),
      groups: HashMap::new(),
      trigger_tags: HashMap::new(),
      top_level: Vec::new(),
    }
  }

  pub fn security_check(self) -> Self {
    let triggers: HashMap<UUID, Trigger> = self
      .triggers
      .into_iter()
      .map(|(id, trigger)| (id, trigger.security_check()))
      .collect();
    Self { triggers, ..self }
  }

  pub fn trigger_count(&self) -> usize {
    self.triggers.len()
  }

  /// When importing, children/parent IDs are expected to be handled by the import logic.
  pub fn import_trigger(&mut self, trigger: Trigger) {
    if trigger.parent_id.is_none() {
      self
        .top_level
        .push(TriggerGroupDescendant::T(trigger.id.clone()));
    }
    self.triggers.insert(trigger.id.clone(), trigger);
  }

  /// When importing, children/parent IDs are expected to be handled by the import logic.
  pub fn import_trigger_group(&mut self, group: TriggerGroup) {
    if group.parent_id.is_none() {
      self
        .top_level
        .push(TriggerGroupDescendant::G(group.id.clone()));
    }
    self.groups.insert(group.id.clone(), group);
  }

  /// Takes an iterator of Tag IDs and returns borrows of all distinct Triggers tagged by any
  /// of the tags. The returned triggers have an arbitrary order (due to underlying HashSet).
  pub fn get_distinct_triggers_tagged_by_any_of<'a, I>(&'a self, tag_ids: I) -> Vec<&'a Trigger>
  where
    I: Iterator<Item = &'a UUID>,
  {
    let distinct_trigger_ids: HashSet<&UUID> = self
      .get_trigger_tags(tag_ids)
      .into_iter()
      .flat_map(|tag| tag.triggers.iter())
      .collect();
    distinct_trigger_ids
      .into_iter()
      .filter_map(|trigger_id| self.triggers.get(trigger_id))
      .collect()
  }

  fn get_trigger_tags<'a, I>(&'a self, tag_ids: I) -> Vec<&'a TriggerTag>
  where
    I: Iterator<Item = &'a UUID>,
  {
    tag_ids.filter_map(|id| self.trigger_tags.get(id)).collect()
  }

  fn try_get_mutable_trigger(
    &mut self,
    trigger_id: &UUID,
  ) -> Result<&mut Trigger, DataMutationError> {
    let Some(trigger) = self.triggers.get_mut(trigger_id) else {
      error!("Tried to mutate Trigger[{trigger_id}] but it was not found!");
      return Err(DataMutationError::TriggerNotFound(trigger_id.to_owned()));
    };
    Ok(trigger)
  }

  pub fn mutate(&mut self, mutation: Mutation) -> MutationResult {
    match mutation {
      Mutation::CreateTrigger {
        trigger,
        parent_position,
      } => {
        let id = trigger.id.clone();
        let parent_delta = if let Some(parent_id) = &trigger.parent_id {
          if let Some(parent) = self.groups.get_mut(parent_id) {
            parent
              .children
              .insert(parent_position, TriggerGroupDescendant::T(id.clone()));
            DataDelta::TriggerGroupChildrenChanged {
              trigger_group_id: parent_id.to_owned(),
              children: parent.children.clone(),
            }
          } else {
            error!(
              "Tried to save a Trigger with an unknown parent! Appending to top-level instead"
            );
            self.top_level.push(TriggerGroupDescendant::T(id.clone()));
            DataDelta::TopLevelChanged(self.top_level.clone())
          }
        } else {
          self
            .top_level
            .insert(parent_position, TriggerGroupDescendant::T(id.clone()));
          DataDelta::TopLevelChanged(self.top_level.clone())
        };
        self.triggers.insert(id, trigger.clone());
        Ok(vec![DataDelta::TriggerSaved(trigger), parent_delta])
      }
      Mutation::SaveTrigger(trigger) => {
        _ = self.try_get_mutable_trigger(&trigger.id)?;
        self.triggers.insert(trigger.id.clone(), trigger.clone());
        Ok(vec![DataDelta::TriggerSaved(trigger)])
      }
      Mutation::SetTriggerName {
        trigger_id,
        new_name,
      } => {
        let trigger = self.try_get_mutable_trigger(&trigger_id)?;
        trigger.name = new_name;
        trigger.updated_now();
        Ok(vec![DataDelta::TriggerSaved(trigger.clone())])
      }
      Mutation::EffectTemplateChanged {
        trigger_id,
        effect_id,
        tmpl,
      } => {
        let trigger = self.try_get_mutable_trigger(&trigger_id)?;
        let effect = trigger
          .get_mut_effect(&effect_id)
          .ok_or_else(|| DataMutationError::EffectNotFound(effect_id))?;
        match effect.inner {
          Effect::CopyToClipboard(_) => effect.inner = Effect::CopyToClipboard(tmpl),
          Effect::Speak { interrupt, .. } => effect.inner = Effect::Speak { tmpl, interrupt },
          // TODO: MORE
          // TODO: MORE
          // TODO: MORE
          // TODO: MORE
          // TODO: MORE
          _ => {}
        }
        trigger.updated_now();
        Ok(vec![DataDelta::TriggerSaved(trigger.clone())])
      }
      Mutation::EffectSpeakInterrupt {
        trigger_id,
        effect_id,
        interrupt,
      } => {
        let trigger = self.try_get_mutable_trigger(&trigger_id)?;
        let effect = trigger
          .get_mut_effect(&effect_id)
          .ok_or_else(|| DataMutationError::EffectNotFound(effect_id))?;
        match &mut effect.inner {
          Effect::Speak { tmpl, .. } => {
            effect.inner = Effect::Speak {
              tmpl: tmpl.to_owned(),
              interrupt,
            };
          }
          _ => return Err(DataMutationError::IncorrectEffectType),
        }
        trigger.updated_now();
        Ok(vec![DataDelta::TriggerSaved(trigger.clone())])
      }
      Mutation::TagTrigger {
        trigger_id,
        trigger_tag_id,
      } => {
        _ = self.try_get_mutable_trigger(&trigger_id)?;
        if let Some(tag) = self.trigger_tags.get_mut(&trigger_tag_id) {
          tag.triggers.insert(trigger_id.clone());
          Ok(vec![DataDelta::TriggerTagged {
            trigger_id: trigger_id.clone(),
            trigger_tag_id,
          }])
        } else {
          error!("Tried to add Trigger[{trigger_id}] to TriggerTag[{trigger_tag_id}], but the TriggerTag does not exist!");
          Err(DataMutationError::TriggerTagNotFound(trigger_tag_id))
        }
      }
      Mutation::UntagTrigger {
        trigger_id,
        trigger_tag_id,
      } => {
        _ = self.try_get_mutable_trigger(&trigger_id)?;
        if let Some(tag) = self.trigger_tags.get_mut(&trigger_tag_id) {
          tag.triggers.remove(&trigger_id);
          Ok(vec![DataDelta::TriggerUntagged {
            trigger_id: trigger_id,
            trigger_tag_id,
          }])
        } else {
          error!("Tried to add Trigger[{trigger_id}] to TriggerTag[{trigger_tag_id}], but the TriggerTag does not exist!");
          Err(DataMutationError::TriggerTagNotFound(trigger_tag_id))
        }
      }
      Mutation::CreateTriggerGroup {
        trigger_group,
        parent_position,
      } => self.create_trigger_group(trigger_group, parent_position),
      Mutation::CreateTriggerTag(name) => Ok(vec![DataDelta::TriggerTagCreated(
        self.create_trigger_tag(&name),
      )]),
      Mutation::DeleteTriggerTag(trigger_tag_id) => self.delete_trigger_tag(&trigger_tag_id),
    }
  }

  pub fn delete_trigger_tag(&mut self, id: &UUID) -> MutationResult {
    self.trigger_tags.remove(id);
    Ok(vec![DataDelta::TriggerTagDeleted(id.to_owned())])
  }

  pub fn create_trigger_tag(&mut self, name: &str) -> TriggerTag {
    let tag = TriggerTag::new(name);
    self.trigger_tags.insert(tag.id.clone(), tag.clone());
    tag
  }

  pub fn create_trigger_group(
    &mut self,
    group: TriggerGroup,
    parent_position: usize,
  ) -> MutationResult {
    let parent_delta: DataDelta = if let Some(parent_id) = &group.parent_id {
      let Some(parent) = self.groups.get_mut(parent_id) else {
        return Err(DataMutationError::TriggerGroupNotFound(parent_id.clone()));
      };
      let index = min(parent_position, parent.children.len());
      parent
        .children
        .insert(index, TriggerGroupDescendant::G(group.id.clone()));
      DataDelta::TriggerGroupChildrenChanged {
        trigger_group_id: parent_id.clone(),
        children: parent.children.clone(),
      }
    } else {
      let index = min(parent_position, self.top_level.len());
      self
        .top_level
        .insert(index, TriggerGroupDescendant::G(group.id.clone()));
      DataDelta::TopLevelChanged(self.top_level.clone())
    };

    self.groups.insert(group.id.clone(), group.clone());

    let group_delta = DataDelta::TriggerGroupCreated(group);

    Ok(vec![group_delta, parent_delta])
  }
}

impl TriggerTag {
  pub fn new(name: &str) -> Self {
    Self {
      id: UUID::new(),
      name: name.to_owned(),
      triggers: HashSet::new(),
    }
  }
}
