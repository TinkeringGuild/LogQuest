use crate::config::LogQuestConfig;
use std::sync::Mutex;

pub struct AppState {
  pub overlay_state: OverlayState,
  pub config: Mutex<LogQuestConfig>,
}

pub struct OverlayState {
  pub overlay_editable: Mutex<bool>,
}

impl AppState {
  pub fn init_from_config(app_config: LogQuestConfig) -> anyhow::Result<AppState> {
    let state = AppState {
      overlay_state: OverlayState::default(),
      config: Mutex::new(app_config),
    };
    Ok(state)
  }
}

impl Default for OverlayState {
  fn default() -> Self {
    Self {
      overlay_editable: Mutex::new(false),
    }
  }
}
