use super::config::LogQuestConfig;
use crate::logs::active_character_detection::Character;
use crate::triggers::TriggerRoot;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use ts_rs::TS;

pub struct StateTree {
  pub config: Mutex<LogQuestConfig>,
  pub reactor: Mutex<ReactorState>,
  pub triggers: Mutex<TriggerRoot>,
  pub overlay: Mutex<OverlayState>,
}

#[derive(TS)]
pub struct ReactorState {
  pub current_character: Option<Character>,
}

#[derive(TS, Clone, Debug, Serialize, Deserialize)]
pub struct OverlayState {
  pub overlay_editable: bool,
}

impl StateTree {
  pub fn init_with_config_and_triggers(
    app_config: LogQuestConfig,
    trigger_root: TriggerRoot,
  ) -> anyhow::Result<StateTree> {
    let state = Self {
      config: Mutex::new(app_config),
      triggers: Mutex::new(trigger_root),
      reactor: Mutex::new(ReactorState::default()),
      overlay: Mutex::new(OverlayState::default()),
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
