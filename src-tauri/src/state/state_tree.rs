use super::config::LogQuestConfig;
use super::overlay::OverlayMode;
use crate::triggers::trigger_index::TriggerIndex;
use crate::{common::UUID, logs::active_character_detection::Character};
use serde::Serialize;
use std::{collections::HashSet, sync::Mutex};

pub const DEFAULT_OVERLAY_OPACITY: u8 = 75;

pub struct StateTree {
  // TODO: SHOULD THESE BE Arc<AsyncRwLock<..> INSTEAD OF Mutex<..> ??
  pub config: Mutex<LogQuestConfig>,
  pub reactor: Mutex<ReactorState>,
  pub triggers: Mutex<TriggerIndex>,
  pub overlay: Mutex<OverlayState>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
pub struct ReactorState {
  pub current_character: Option<Character>,
  pub active_trigger_tags: HashSet<UUID>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
pub struct OverlayState {
  pub overlay_editable: bool,

  /// stored as an integer representing percentage. Valid values are 0-100
  pub overlay_opacity: u8,

  #[ts(as = "Option<_>")]
  pub overlay_mode: OverlayMode,

  #[ts(skip)]
  #[serde(skip)]
  pub auto_open_dev_tools: bool,
}

impl StateTree {
  pub fn new(
    app_config: LogQuestConfig,
    trigger_index: TriggerIndex,
    overlay_mode: OverlayMode,
    overlay_dev_tools: bool,
  ) -> StateTree {
    Self {
      config: Mutex::new(app_config),
      triggers: Mutex::new(trigger_index),
      reactor: Mutex::new(ReactorState::new()),
      overlay: Mutex::new(OverlayState::new(overlay_mode, overlay_dev_tools)),
    }
  }
}

impl ReactorState {
  fn new() -> Self {
    Self {
      current_character: None,
      active_trigger_tags: HashSet::new(),
    }
  }
}

impl OverlayState {
  fn new(overlay_mode: OverlayMode, auto_open_dev_tools: bool) -> Self {
    Self {
      overlay_editable: false,
      overlay_opacity: DEFAULT_OVERLAY_OPACITY,
      overlay_mode,
      auto_open_dev_tools,
    }
  }
}
