use super::config::LogQuestConfig;
use crate::logs::active_character_detection::Character;
use crate::triggers::TriggerGroup;
use std::sync::Mutex;

pub struct StateTree {
  pub overlay: Mutex<OverlayState>,
  pub reactor: Mutex<ReactorState>,
  pub config: Mutex<LogQuestConfig>,
  pub triggers: Mutex<Vec<TriggerGroup>>,
}

pub struct ReactorState {
  pub current_character: Option<Character>,
}

pub struct OverlayState {
  pub overlay_editable: bool,
}

impl StateTree {
  pub fn init_from_configs(
    app_config: LogQuestConfig,
    triggers: Vec<TriggerGroup>,
  ) -> anyhow::Result<StateTree> {
    let state = Self {
      triggers: Mutex::new(triggers),
      overlay: Mutex::new(OverlayState::default()),
      reactor: Mutex::new(ReactorState::default()),
      config: Mutex::new(app_config),
    };
    Ok(state)
  }
}

impl Default for ReactorState {
  fn default() -> Self {
    Self {
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
