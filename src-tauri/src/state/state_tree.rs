use super::config::LogQuestConfig;
use crate::{
  debug_only::test_trigger_group, logs::active_character_detection::Character,
  triggers::TriggerGroup,
};
use std::sync::Mutex;

pub struct StateTree {
  pub overlay_state: Mutex<OverlayState>,
  pub reactor_state: Mutex<ReactorState>,
  pub config: Mutex<LogQuestConfig>,
}

pub struct ReactorState {
  pub trigger_groups: Vec<TriggerGroup>,
  pub current_character: Option<Character>,
}

pub struct OverlayState {
  pub overlay_editable: bool,
}

impl StateTree {
  pub fn init_from_config(app_config: LogQuestConfig) -> anyhow::Result<StateTree> {
    let state = Self {
      overlay_state: Mutex::new(OverlayState::default()),
      reactor_state: Mutex::new(ReactorState::default()),
      config: Mutex::new(app_config),
    };
    Ok(state)
  }
}

impl Default for ReactorState {
  fn default() -> Self {
    Self {
      trigger_groups: vec![test_trigger_group()],
      current_character: None,
    }
  }
}

impl Default for OverlayState {
  fn default() -> Self {
    Self {
      overlay_editable: false,
    }
  }
}
