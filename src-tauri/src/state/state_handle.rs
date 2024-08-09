use super::config::LogQuestConfig;
use super::state_tree::{OverlayState, ReactorState, StateTree};
use crate::triggers::TriggerRoot;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// `StateHandle` provides helper methods for accessing `Mutex`-locked branches
/// of the `StateTree`. There are three different ways to access a branch...
///
/// 1. `with_B()` - provides immutable access to the branch value
/// 2. `select_B()` - utility to extract a value held within a branch
/// 3. `update_B()` - provides mutable access; may automatically persist changes
///
/// ...where `B` is the name of the `Mutex` field in `StateTree`.
///
/// `StateHandle` is also safe to `clone` since it wraps the `StateTree` in an `Arc`
/// and all of its fields are `Mutex`-guarded.
#[derive(Clone)]
pub struct StateHandle(Arc<StateTree>);

impl StateHandle {
  pub fn new(state_tree: StateTree) -> Self {
    Self(Arc::new(state_tree))
  }

  pub fn with_config<F>(&self, reader: F)
  where
    F: FnOnce(&LogQuestConfig),
  {
    self.with_branch(&self.0.config, reader);
  }

  pub fn with_reactor<F>(&self, reader: F)
  where
    F: FnOnce(&ReactorState),
  {
    self.with_branch(&self.0.reactor, reader);
  }

  pub fn with_triggers<F>(&self, reader: F)
  where
    F: FnOnce(&TriggerRoot),
  {
    self.with_branch(&self.0.triggers, reader);
  }

  pub fn select_config<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&LogQuestConfig) -> T,
  {
    self.select_branch(&self.0.config, selector)
  }

  pub fn select_overlay<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&OverlayState) -> T,
  {
    self.select_branch(&self.0.overlay, selector)
  }

  pub fn select_triggers<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&TriggerRoot) -> T,
  {
    self.select_branch(&self.0.triggers, selector)
  }

  pub fn update_reactor<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut ReactorState),
  {
    self.update_branch(&self.0.reactor, func);
  }

  pub fn update_triggers<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut TriggerRoot),
  {
    self.update_branch(&self.0.triggers, |root| {
      func(root); // mutates root
      self.with_config(|config| {
        if let Err(e) = config.save_triggers(root) {
          error!("COULD NOT SAVE TRIGGERS TO DISK! [ ERROR: {e:?} ]");
        }
      });
    });
  }

  pub fn update_overlay<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut OverlayState),
  {
    self.update_branch(&self.0.overlay, func);
  }

  /// Automatically saves the config if a change is detected
  pub fn update_config<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut LogQuestConfig),
  {
    self.update_branch(&self.0.config, |config| {
      let config_before = config.clone();
      func(config);
      if config_before != *config {
        info!(
          "Saving config to {}",
          config_before.config_file_path.display()
        );
        if let Err(e) = config.save_config() {
          error!("Could not save config! {e:?}");
        }
      }
    })
  }

  fn with_branch<F, B>(&self, branch: &Mutex<B>, reader: F)
  where
    F: FnOnce(&B),
  {
    let guard = branch.lock().expect("State mutex poisoned!");
    let value: &B = &*guard;
    reader(value);
  }

  fn select_branch<F, B, T>(&self, branch: &Mutex<B>, func: F) -> T
  where
    F: FnOnce(&B) -> T,
  {
    let mut guard = branch.lock().expect("State mutex poisoned!");
    let value: &mut B = &mut *guard;
    func(value)
  }

  fn update_branch<F, B>(&self, branch: &Mutex<B>, func: F)
  where
    F: for<'a> FnOnce(&'a mut B),
  {
    let mut guard = branch.lock().expect("State mutex poisoned!");
    let value: &mut B = &mut *guard;
    func(value);
  }
}
