use super::config::LogQuestConfig;
use crate::logs::active_character_detection::Character;
use crate::triggers::TriggerRoot;
use std::sync::Mutex;

pub struct StateTree {
  pub overlay: Mutex<OverlayState>,
  pub reactor: Mutex<ReactorState>,
  pub config: Mutex<LogQuestConfig>,
  pub triggers: Mutex<TriggerRoot>,
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
    trigger_root: TriggerRoot,
  ) -> anyhow::Result<StateTree> {
    let state = Self {
      triggers: Mutex::new(trigger_root),
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
