use super::{Trigger, TriggerGroup};
use crate::common::{LogQuestVersion, UUID};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::iter::once;
use std::{cmp::min, collections::HashSet};
use tracing::error;

fn is_compatible_triggers_import_version(_version: &LogQuestVersion) -> bool {
  true // TODO: This will need to be maintained over time
}

pub type MutationResult = Result<Vec<DataDelta>, DataMutationError>;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, ts_rs::TS)]
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
  CreateTrigger {
    trigger: Trigger,
    trigger_tag_ids: Vec<UUID>,
    parent_position: usize,
  },
  SaveTrigger {
    trigger: Trigger,
    trigger_tag_ids: Vec<UUID>,
  },
  DeleteTrigger(UUID),
  CreateTriggerGroup {
    trigger_group: TriggerGroup,
    parent_position: usize,
  },
  SaveTriggerGroup {
    trigger_group_id: UUID,
    name: String,
    comment: Option<String>,
  },
  DeleteTriggerGroup(UUID),
  CreateTriggerTag(String),
  RenameTriggerTag(UUID, String),
  DeleteTriggerTag(UUID),
  TagTrigger {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  UntagTrigger {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum DataDelta {
  TopLevelChanged(Vec<TriggerGroupDescendant>),

  TriggerSaved(Trigger),
  TriggerDeleted(UUID),

  TriggerTagged {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },
  TriggerUntagged {
    trigger_id: UUID,
    trigger_tag_id: UUID,
  },

  TriggerGroupSaved(TriggerGroup),
  TriggerGroupChildrenChanged {
    trigger_group_id: UUID,
    children: Vec<TriggerGroupDescendant>,
  },
  TriggerGroupDeleted(UUID),

  TriggerTagCreated(TriggerTag),
  TriggerTagRenamed(UUID, String),
  TriggerTagTriggersChanged {
    trigger_tag_id: UUID,
    triggers: Vec<UUID>,
  },
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
        mut trigger,
        trigger_tag_ids,
        parent_position,
      } => {
        trigger.updated_now();

        let trigger_id = trigger.id.clone();
        self.triggers.insert(trigger_id.clone(), trigger.clone());

        let parent_delta = if let Some(parent_id) = &trigger.parent_id {
          if let Some(parent) = self.groups.get_mut(parent_id) {
            parent.children.insert(
              parent_position,
              TriggerGroupDescendant::T(trigger_id.clone()),
            );
            DataDelta::TriggerGroupChildrenChanged {
              trigger_group_id: parent_id.to_owned(),
              children: parent.children.clone(),
            }
          } else {
            error!(
              "Tried to save a Trigger with an unknown parent! Appending to top-level instead"
            );
            self
              .top_level
              .push(TriggerGroupDescendant::T(trigger_id.clone()));
            DataDelta::TopLevelChanged(self.top_level.clone())
          }
        } else {
          self.top_level.insert(
            parent_position,
            TriggerGroupDescendant::T(trigger_id.clone()),
          );
          DataDelta::TopLevelChanged(self.top_level.clone())
        };

        let mut deltas = vec![DataDelta::TriggerSaved(trigger), parent_delta];
        for tag_id in trigger_tag_ids.into_iter() {
          if let Some(trigger_tag) = self.trigger_tags.get_mut(&tag_id) {
            trigger_tag.triggers.insert(trigger_id.clone());
            deltas.push(DataDelta::TriggerTagged {
              trigger_id: trigger_id.clone(),
              trigger_tag_id: tag_id,
            });
          }
        }

        Ok(deltas)
      }
      Mutation::SaveTrigger {
        mut trigger,
        trigger_tag_ids,
      } => {
        _ = self.try_get_mutable_trigger(&trigger.id)?;
        trigger.updated_now();

        let trigger_id = trigger.id.clone();

        let replaced_trigger_maybe = self.triggers.insert(trigger.id.clone(), trigger.clone());
        let parent_delta_maybe: Option<DataDelta> =
          replaced_trigger_maybe.and_then(|replaced_trigger| {
            if replaced_trigger.parent_id == trigger.parent_id {
              return None;
            }
            let different_tgd = |tgd: &TriggerGroupDescendant| match tgd {
              TriggerGroupDescendant::T(tgd_id) => tgd_id != &trigger.id,
              _ => true,
            };
            if let Some(previous_parent_id) = replaced_trigger.parent_id {
              let Some(parent_group) = self.groups.get_mut(&previous_parent_id) else {
                return None;
              };
              parent_group.children.retain(different_tgd);
              Some(DataDelta::TriggerGroupChildrenChanged {
                trigger_group_id: parent_group.id.clone(),
                children: parent_group.children.clone(),
              })
            } else {
              self.top_level.retain(different_tgd);
              Some(DataDelta::TopLevelChanged(self.top_level.clone()))
            }
          });

        let mut deltas = vec![DataDelta::TriggerSaved(trigger)];

        if let Some(parent_delta) = parent_delta_maybe {
          deltas.push(parent_delta);
        }

        let new_trigger_tag_ids: HashSet<UUID> = HashSet::from_iter(trigger_tag_ids.into_iter());

        for each_trigger_tag in self.trigger_tags.values_mut() {
          if each_trigger_tag.triggers.contains(&trigger_id) {
            if !new_trigger_tag_ids.contains(&each_trigger_tag.id) {
              each_trigger_tag.triggers.remove(&trigger_id);
              deltas.push(DataDelta::TriggerUntagged {
                trigger_id: trigger_id.clone(),
                trigger_tag_id: each_trigger_tag.id.clone(),
              });
            }
          } else if new_trigger_tag_ids.contains(&each_trigger_tag.id) {
            each_trigger_tag.triggers.insert(trigger_id.clone());
            deltas.push(DataDelta::TriggerTagged {
              trigger_id: trigger_id.clone(),
              trigger_tag_id: each_trigger_tag.id.clone(),
            });
          }
        }

        Ok(deltas)
      }
      Mutation::DeleteTrigger(trigger_id) => {
        let Some(trigger) = self.triggers.remove(&trigger_id) else {
          return Err(DataMutationError::TriggerNotFound(trigger_id));
        };
        let parent_delta = if let Some(parent_id) = trigger.parent_id {
          if let Some(parent) = self.groups.get_mut(&parent_id) {
            remove_trigger_from_descendants(&trigger_id, &mut parent.children);
            Some(DataDelta::TriggerGroupChildrenChanged {
              trigger_group_id: parent.id.clone(),
              children: parent.children.clone(),
            })
          } else {
            remove_trigger_from_descendants(&trigger_id, &mut self.top_level);
            Some(DataDelta::TopLevelChanged(self.top_level.clone()))
          }
        } else {
          error!("Deleting Trigger[{trigger_id}] but its parent was not found!");
          None
        };

        let trigger_tag_deltas: Vec<DataDelta> = self
          .mut_trigger_tags_with_trigger(&trigger_id)
          .into_iter()
          .map(|tag| {
            tag.triggers.remove(&trigger_id);
            DataDelta::TriggerTagTriggersChanged {
              trigger_tag_id: trigger_id.clone(),
              triggers: tag.triggers.iter().cloned().collect(),
            }
          })
          .collect();

        let deltas = once(DataDelta::TriggerDeleted(trigger_id))
          .chain(parent_delta.into_iter())
          .chain(trigger_tag_deltas.into_iter())
          .collect();

        Ok(deltas)
      }
      Mutation::SaveTriggerGroup {
        trigger_group_id,
        name,
        comment,
      } => {
        let Some(group) = self.groups.get_mut(&trigger_group_id) else {
          return Err(DataMutationError::TriggerGroupNotFound(trigger_group_id));
        };
        group.name = name;
        group.comment = comment;
        Ok(vec![DataDelta::TriggerGroupSaved(group.clone())])
      }
      Mutation::DeleteTriggerGroup(group_id) => {
        let Some(group) = self.groups.remove(&group_id) else {
          return Err(DataMutationError::TriggerGroupNotFound(group_id));
        };

        let parent_delta = if let Some(parent_id) = group.parent_id {
          if let Some(parent) = self.groups.get_mut(&parent_id) {
            remove_group_from_descendants(&group_id, &mut parent.children);
            Some(DataDelta::TriggerGroupChildrenChanged {
              trigger_group_id: parent.id.clone(),
              children: parent.children.clone(),
            })
          } else {
            error!("Deleting TriggerGroup[{group_id}] but its parent was not found!");
            None
          }
        } else {
          remove_group_from_descendants(&group_id, &mut self.top_level);
          Some(DataDelta::TopLevelChanged(self.top_level.clone()))
        };

        let mut descendants: VecDeque<TriggerGroupDescendant> = group.children.into();
        let mut nested_triggers = VecDeque::<Trigger>::new();
        let mut nested_groups = VecDeque::<TriggerGroup>::new();

        while let Some(descendant) = descendants.pop_front() {
          match descendant {
            TriggerGroupDescendant::G(descendant_group_id) => {
              if let Some(mut descendant_group) = self.groups.remove(&descendant_group_id) {
                descendants.append(&mut VecDeque::from_iter(
                  descendant_group.children.drain(..).into_iter(),
                ));
                nested_groups.push_back(descendant_group);
              }
            }
            TriggerGroupDescendant::T(descendant_trigger_id) => {
              if let Some(descendant_trigger) = self.triggers.remove(&descendant_trigger_id) {
                nested_triggers.push_back(descendant_trigger);
              }
            }
          }
        }

        let updated_trigger_tags: HashSet<UUID> = nested_triggers
          .iter()
          .flat_map(|trigger| {
            self
              .trigger_tags_with_trigger(&trigger.id)
              .into_iter()
              .map(|tag| tag.id.clone())
          })
          .collect();

        let removed_trigger_ids: HashSet<&UUID> = nested_triggers.iter().map(|t| &t.id).collect();
        let trigger_tag_deltas: Vec<DataDelta> = updated_trigger_tags
          .into_iter()
          .map(|tag_id| {
            let tag = self.trigger_tags.get_mut(&tag_id).unwrap();
            tag
              .triggers
              .retain(|trigger_id| !removed_trigger_ids.contains(trigger_id));
            DataDelta::TriggerTagTriggersChanged {
              trigger_tag_id: tag.id.clone(),
              triggers: tag.triggers.iter().cloned().collect(),
            }
          })
          .collect();

        let trigger_deltas: Vec<DataDelta> = nested_triggers
          .into_iter()
          .map(|trigger| DataDelta::TriggerDeleted(trigger.id))
          .collect();

        let trigger_group_deltas: Vec<DataDelta> = nested_groups
          .into_iter()
          .map(|group| DataDelta::TriggerGroupDeleted(group.id))
          .collect();

        let deltas: Vec<DataDelta> = once(DataDelta::TriggerGroupDeleted(group_id))
          .chain(parent_delta.into_iter())
          .chain(trigger_deltas.into_iter())
          .chain(trigger_group_deltas.into_iter())
          .chain(trigger_tag_deltas.into_iter())
          .collect();

        Ok(deltas)
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
      Mutation::RenameTriggerTag(trigger_tag_id, name) => {
        let Some(trigger_tag) = self.trigger_tags.get_mut(&trigger_tag_id) else {
          return Err(DataMutationError::TriggerTagNotFound(trigger_tag_id));
        };
        trigger_tag.name = name.clone();
        Ok(vec![DataDelta::TriggerTagRenamed(trigger_tag_id, name)])
      }
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

    let group_delta = DataDelta::TriggerGroupSaved(group);

    Ok(vec![group_delta, parent_delta])
  }

  fn trigger_tags_with_trigger(&self, trigger_id: &UUID) -> Vec<&TriggerTag> {
    self
      .trigger_tags
      .iter()
      .filter_map(|(_, tag)| {
        if tag.triggers.contains(trigger_id) {
          Some(tag)
        } else {
          None
        }
      })
      .collect()
  }

  fn mut_trigger_tags_with_trigger(&mut self, trigger_id: &UUID) -> Vec<&mut TriggerTag> {
    self
      .trigger_tags
      .iter_mut()
      .filter_map(|(_, tag)| {
        if tag.triggers.contains(trigger_id) {
          Some(tag)
        } else {
          None
        }
      })
      .collect()
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

impl std::hash::Hash for TriggerTag {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state)
  }
}

impl PartialEq for TriggerTag {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

fn remove_trigger_from_descendants(
  trigger_id: &UUID,
  descendants: &mut Vec<TriggerGroupDescendant>,
) {
  let trigger_index = descendants.iter().position(|tgd_trigger_id| {
    if let TriggerGroupDescendant::T(tgd_id) = tgd_trigger_id {
      tgd_id == trigger_id
    } else {
      false
    }
  });
  if let Some(trigger_index) = trigger_index {
    descendants.remove(trigger_index);
  }
}

fn remove_group_from_descendants(group_id: &UUID, descendants: &mut Vec<TriggerGroupDescendant>) {
  let group_index = descendants.iter().position(|tgd| {
    if let TriggerGroupDescendant::G(tgd_group_id) = tgd {
      tgd_group_id == group_id
    } else {
      false
    }
  });
  if let Some(group_index) = group_index {
    descendants.remove(group_index);
  }
}
