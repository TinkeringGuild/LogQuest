use super::config::{LogQuestConfig, TriggersSaveError};
use super::state_tree::{OverlayState, ReactorState, StateTree};
use crate::common::shutdown::critical_path;
use crate::triggers::trigger_index::{DataDelta, DataMutationError, TriggerIndex};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
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
/// `StateHandle` is safe to `clone` since it wraps its fields with `Arc`s and
/// and all of the `StateTree` fields are `Mutex`-guarded.
#[derive(Clone)]
pub struct StateHandle {
  tree: Arc<StateTree>,
  pub config_updated: Arc<Notify>,
}

impl StateHandle {
  pub fn new(state_tree: StateTree) -> Self {
    let config_updated = Arc::new(Notify::new());
    Self {
      tree: Arc::new(state_tree),
      config_updated,
    }
  }

  pub fn with_config<F>(&self, reader: F)
  where
    F: FnOnce(&LogQuestConfig),
  {
    self.with_branch(&self.tree.config, reader);
  }

  pub fn with_reactor<F>(&self, reader: F)
  where
    F: FnOnce(&ReactorState),
  {
    self.with_branch(&self.tree.reactor, reader);
  }

  // pub fn with_triggers<F>(&self, reader: F)
  // where
  //   F: FnOnce(&TriggerIndex),
  // {
  //   self.with_branch(&self.tree.triggers, reader);
  // }

  pub fn select_config<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&LogQuestConfig) -> T,
  {
    self.select_branch(&self.tree.config, selector)
  }

  pub fn select_overlay<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&OverlayState) -> T,
  {
    self.select_branch(&self.tree.overlay, selector)
  }

  pub fn select_triggers<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&TriggerIndex) -> T,
  {
    self.select_branch(&self.tree.triggers, selector)
  }

  pub fn select_reactor<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&ReactorState) -> T,
  {
    self.select_branch(&self.tree.reactor, selector)
  }

  pub fn update_reactor_and_select<F, T>(&self, selector: F) -> T
  where
    F: FnOnce(&mut ReactorState) -> T,
  {
    self.update_branch_and_select(&self.tree.reactor, selector)
  }

  pub fn update_reactor<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut ReactorState),
  {
    self.update_branch(&self.tree.reactor, func);
  }

  pub fn mutate_index<F>(&self, func: F) -> Result<Vec<DataDelta>, TriggersSaveError>
  where
    F: FnOnce(&mut TriggerIndex) -> Result<Vec<DataDelta>, DataMutationError>,
  {
    self.update_branch_and_select(&self.tree.triggers, |index| {
      self.select_config(|config| {
        critical_path(|| {
          let deltas = func(index)?;
          for delta in deltas.iter() {
            match delta {
              DataDelta::TriggerSaved(trigger) => config.save_trigger(&trigger)?,
              DataDelta::TriggerDeleted(trigger_id) => {
                config.delete_trigger_file(trigger_id)?;
              }
              DataDelta::TriggerTagCreated(trigger_tag) => {
                config.save_trigger_tag(trigger_tag)?;
              }
              DataDelta::TriggerTagDeleted(trigger_tag_id) => {
                config.delete_trigger_tag_file(trigger_tag_id)?;
              }
              DataDelta::TriggerTagged { trigger_tag_id, .. }
              | DataDelta::TriggerUntagged { trigger_tag_id, .. }
              | DataDelta::TriggerTagTriggersChanged { trigger_tag_id, .. } => {
                if let Some(trigger_tag) = index.trigger_tags.get(trigger_tag_id) {
                  config.save_trigger_tag(trigger_tag)?;
                }
              }
              DataDelta::TriggerGroupSaved(group) => {
                config.save_trigger_group(group)?;
              }
              DataDelta::TriggerGroupDeleted(group_id) => {
                config.delete_trigger_group_file(group_id)?;
              }
              DataDelta::TriggerGroupChildrenChanged {
                trigger_group_id, ..
              } => {
                if let Some(group) = index.groups.get(trigger_group_id) {
                  config.save_trigger_group(group)?;
                }
              }
              DataDelta::TopLevelChanged(top_level) => {
                config.save_top_level(top_level)?;
              }
            }
          }
          Ok(deltas)
        })
      })
    })
  }

  pub fn bulk_update_triggers<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut TriggerIndex),
  {
    // Does not release the lock on the triggers until the JSON serialization
    // has been fully flushed to disk.
    self.update_branch(&self.tree.triggers, |index| {
      func(index); // mutates index
      self.with_config(|config| {
        if let Err(e) = config.save_trigger_index(index) {
          error!("COULD NOT SAVE TRIGGERS TO DISK! [ ERROR: {e:?} ]");
        }
      });
    });
  }

  pub fn update_overlay<F>(&self, func: F)
  where
    F: for<'a> FnOnce(&'a mut OverlayState),
  {
    self.update_branch(&self.tree.overlay, func);
  }

  // /// Automatically saves the config if a change is detected
  // pub fn update_config<F, R>(&self, func: F)
  // where
  //   F: FnOnce(&mut LogQuestConfig),
  // {
  //   self.update_config_and_select(func);
  // }

  /// Automatically saves the config if a change is detected
  pub fn update_config_and_select<F, R>(&self, func: F) -> R
  where
    F: FnOnce(&mut LogQuestConfig) -> R,
  {
    self.update_branch_and_select(&self.tree.config, |config| {
      let config_before = config.clone();
      let returned = func(config);
      if config_before != *config {
        info!(
          "Saving config to {}",
          config_before.config_file_path.display()
        );
        if let Err(e) = config.save_config() {
          error!("Could not save config! {e:?}");
        }
        self.config_updated.notify_waiters();
      }
      returned
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
    F: FnOnce(&mut B),
  {
    self.update_branch_and_select(branch, |b: &mut B| {
      func(b);
      ()
    });
  }

  fn update_branch_and_select<F, B, R>(&self, branch: &Mutex<B>, func: F) -> R
  where
    F: FnOnce(&mut B) -> R,
  {
    let mut guard = branch.lock().expect("State mutex poisoned!");
    let value: &mut B = &mut *guard;
    func(value)
  }
}
