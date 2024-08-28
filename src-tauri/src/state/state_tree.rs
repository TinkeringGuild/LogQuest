use super::config::LogQuestConfig;
use super::overlay::OverlayMode;
use crate::logs::active_character_detection::Character;
use crate::triggers::TriggerRoot;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct StateTree {
  // TODO: SHOULD THESE BE Arc<RwLock<..> INSTEAD OF Mutex<..> ??
  pub config: Mutex<LogQuestConfig>,
  pub reactor: Mutex<ReactorState>,
  pub triggers: Mutex<TriggerRoot>,
  pub overlay: Mutex<OverlayState>,
}

#[derive(ts_rs::TS)]
pub struct ReactorState {
  pub current_character: Option<Character>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
pub struct OverlayState {
  pub overlay_editable: bool,
  #[ts(as = "Option<_>")]
  pub overlay_mode: OverlayMode,
  #[ts(skip)]
  #[serde(skip)]
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
