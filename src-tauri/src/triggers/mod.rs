pub mod command_template;
pub mod effects;
pub mod template_string;
pub mod timers;
pub mod trigger_index;

use crate::{
  common::{timestamp::Timestamp, UUID},
  matchers,
  state::config::{LogQuestConfig, TriggerLoadError, TriggersSaveError},
};
use effects::{Effect, EffectWithID};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use template_string::TemplateString;
use trigger_index::{TriggerGroupDescendant, TriggerIndex};

#[derive(thiserror::Error, Debug)]
pub enum TriggerLoadOrCreateError {
  // #[error(transparent)]
  // IOError(#[from] std::io::Error),
  // #[error(transparent)]
  // DeserializationError(#[from] serde_json::Error),
  #[error(transparent)]
  SaveError(#[from] TriggersSaveError),
  #[error(transparent)]
  LoadError(#[from] TriggerLoadError),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
pub struct TriggerGroup {
  pub id: UUID,
  pub parent_id: Option<UUID>,
  pub name: String,
  pub comment: Option<String>,
  pub children: Vec<TriggerGroupDescendant>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct Trigger {
  pub id: UUID,
  pub parent_id: Option<UUID>,
  pub name: String,
  pub comment: Option<String>,
  pub enabled: bool,
  pub filter: matchers::Filter,
  pub effects: Vec<EffectWithID>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp, // tags: Vec<Tag>
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

  fn get_mut_effect(&mut self, effect_id: &UUID) -> Option<&mut EffectWithID> {
    let mut queue: VecDeque<&mut Vec<EffectWithID>> = [&mut self.effects].into();
    while let Some(effects) = queue.pop_front() {
      for effect in effects.iter_mut() {
        if effect.id == *effect_id {
          return Some(effect);
        }
        match &mut effect.inner {
          Effect::Sequence(effects) | Effect::Parallel(effects) => {
            queue.push_back(effects);
          }
          Effect::StartTimer(timer) => {
            queue.push_back(&mut timer.effects);
          }
          Effect::StartStopwatch(stopwatch) => {
            queue.push_back(&mut stopwatch.effects);
          }
          _ => {}
        }
      }
    }
    None
  }

  fn updated_now(&mut self) {
    self.updated_at = Timestamp::now();
  }
}

pub fn load_or_create_relative_to_config(
  config: &LogQuestConfig,
) -> Result<TriggerIndex, TriggerLoadOrCreateError> {
  if let Some(top_level) = config.load_top_level_file()? {
    let triggers = config
      .load_all_triggers()?
      .into_iter()
      .map(|t| (t.id.clone(), t))
      .collect();

    let groups = config
      .load_all_trigger_groups()?
      .into_iter()
      .map(|g| (g.id.clone(), g))
      .collect();

    let trigger_tags = config
      .load_all_trigger_tags()?
      .into_iter()
      .map(|tag| (tag.id.clone(), tag))
      .collect();

    let index = TriggerIndex {
      triggers,
      groups,
      trigger_tags,
      top_level,
    };
    Ok(index.security_check())
  } else {
    let index = default_triggers().security_check();
    config.save_trigger_index(&index)?;
    Ok(index)
  }
}

#[cfg(not(debug_assertions))]
pub fn default_triggers() -> TriggerIndex {
  TriggerIndex::new()
}

#[cfg(debug_assertions)]
pub fn default_triggers() -> TriggerIndex {
  crate::debug_only::test_trigger_index()
}

#[cfg(test)]
mod test {
  use super::{
    effects::Effect, trigger_index::TriggerGroupDescendant, EffectWithID, Trigger, TriggerGroup,
  };
  use crate::{
    common::{timestamp::Timestamp, UUID},
    matchers::Matcher,
  };

  #[test]
  fn test_serde() {
    let (trigger_before, group_before) = simple_sample();

    {
      let raw_json = serde_json::to_string_pretty(&trigger_before)
        .expect("Could not convert TriggerGroup to JSON");
      let trigger_after: Trigger =
        serde_json::from_str(&raw_json).expect("Could not parse TriggerGroup JSON!");
      assert_eq!(trigger_before, trigger_after);
    }

    {
      let raw_json = serde_json::to_string_pretty(&group_before)
        .expect("Could not convert TriggerGroup to JSON");
      let group_after: TriggerGroup =
        serde_json::from_str(&raw_json).expect("Could not parse TriggerGroup JSON!");
      assert_eq!(group_before, group_after);
    }
  }

  fn simple_sample() -> (Trigger, TriggerGroup) {
    let now = Timestamp::now();
    let trigger_id = UUID::new();
    let group_id = UUID::new();
    let trigger = Trigger {
      id: trigger_id.clone(),
      parent_id: Some(group_id.clone()),
      name: "Simple Sample Trigger 1".into(),
      enabled: true,
      comment: None,
      created_at: now.clone(),
      updated_at: now.clone(),
      filter: vec![Matcher::gina("^{S1} hits {S2}").unwrap()].into(),
      effects: vec![EffectWithID::new(Effect::Sequence(vec![
        EffectWithID::new(Effect::Speak {
          tmpl: "This is only a test.".into(),
          interrupt: false,
        }),
        EffectWithID::new(Effect::PlayAudioFile(Some("/dev/null".into()))),
      ]))],
    };
    let group = TriggerGroup {
      id: group_id,
      parent_id: None,
      name: "Simple Sample Trigger Group".into(),
      comment: Some("I am a comment".into()),
      created_at: now.clone(),
      updated_at: now,
      children: vec![TriggerGroupDescendant::T(trigger_id)],
    };

    (trigger, group)
  }
}
