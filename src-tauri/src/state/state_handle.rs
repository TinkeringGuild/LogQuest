use tracing::{error, info};

use super::config::LogQuestConfig;
use super::state_tree::{OverlayState, ReactorState, StateTree};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StateHandle(Arc<StateTree>);

impl StateHandle {
  pub fn new(state_tree: StateTree) -> Self {
    Self(Arc::new(state_tree))
  }

  pub fn select_config<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&LogQuestConfig) -> &T,
    T: Clone,
  {
    self.select_branch(&self.0.config, selector)
  }

  pub fn select_overlay<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&OverlayState) -> &T,
    T: Clone,
  {
    self.select_branch(&self.0.overlay_state, selector)
  }
  pub fn with_reactor<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut ReactorState),
  {
    self.with_branch(&self.0.reactor_state, func)
  }

  pub fn with_overlay<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut OverlayState),
  {
    self.with_branch(&self.0.overlay_state, func)
  }

  /// Automatically saves the config if a change is detected
  pub fn with_config<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut LogQuestConfig),
  {
    self.with_branch(&self.0.config, |config| {
      let config_before = config.clone();
      func(config);
      if config_before != *config {
        info!(
          "Saving config to {}",
          config_before.config_file_path.display()
        );
        if let Err(e) = config.save() {
          error!("Could not save config! {e:?}");
        }
      }
    })
  }

  fn select_branch<F, B, T>(&self, branch: &Mutex<B>, func: F) -> T
  where
    F: FnOnce(&B) -> &T,
    T: Clone,
  {
    let mut guard = branch.lock().unwrap();
    let value: &mut B = &mut *guard;
    func(value).clone()
  }

  fn with_branch<F, B>(&self, branch: &Mutex<B>, func: F)
  where
    F: for<'a> FnOnce(&'a mut B),
  {
    let mut guard = branch.lock().unwrap();
    let value: &mut B = &mut *guard;
    func(value);
  }
}
