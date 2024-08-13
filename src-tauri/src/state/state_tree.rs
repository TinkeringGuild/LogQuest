use super::config::LogQuestConfig;
use crate::logs::active_character_detection::Character;
use crate::triggers::TriggerRoot;
use crate::ui::OverlayMode;
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
  pub overlay_mode: OverlayMode,
  pub auto_open_dev_tools: bool,
}

impl StateTree {
  pub fn new(
    app_config: LogQuestConfig,
    trigger_root: TriggerRoot,
    overlay_mode: OverlayMode,
    overlay_dev_tools: bool,
  ) -> StateTree {
    Self {
      config: Mutex::new(app_config),
      triggers: Mutex::new(trigger_root),
      reactor: Mutex::new(ReactorState::new()),
      overlay: Mutex::new(OverlayState::new(overlay_mode, overlay_dev_tools)),
    }
  }
}

impl ReactorState {
  fn new() -> Self {
    Self {
      current_character: None,
    }
  }
}

impl OverlayState {
  fn new(overlay_mode: OverlayMode, auto_open_dev_tools: bool) -> Self {
    Self {
      overlay_editable: false,
      overlay_mode,
      auto_open_dev_tools,
    }
  }
}
