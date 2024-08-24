use super::{effects::Effect, template_string::TemplateString};
use crate::{
  common::{duration::Duration, UUID},
  matchers::FilterWithContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct Timer {
  pub trigger_id: UUID,
  pub name_tmpl: TemplateString,
  pub tags: Vec<TimerTag>,
  pub duration: Duration,
  pub start_policy: TimerStartPolicy,

  /// When finished, the timer starts over until terminated
  pub repeats: bool,

  pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
pub enum TimerEffect {
  HideTimer,
  RestartTimer,
  IncrementCounter,
  DecrementCounter,
  ResetCounter,
  AddTag(TimerTag),
  RemoveTag(TimerTag),
  WaitUntilTagged(TimerTag),
  WaitUntilSecondsRemain(u32),
  WaitUntilFilterMatches(FilterWithContext),
  // WaitUntilRestarted,
  WaitUntilFinished,
  ClearTimer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub enum TimerStartPolicy {
  AlwaysStartNewTimer,
  DoNothingIfTimerRunning,
  StartAndReplacesAllTimersOfTrigger,
  StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(TemplateString),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct Stopwatch {
  pub name: TemplateString,
  pub tags: Vec<TimerTag>,
  pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct TimerTag(String);
impl TimerTag {
  pub fn new(name: &str) -> Self {
    Self(name.to_owned())
  }

  /// used for marking a Timer has entered the "ending" state
  pub fn ending() -> Self {
    Self::new("ENDING")
  }
}

impl Timer {
  pub(super) fn security_check(self) -> Self {
    let effects = self
      .effects
      .into_iter()
      .map(|e| e.security_check())
      .collect();
    Self { effects, ..self }
  }
}

impl Stopwatch {
  pub(super) fn security_check(self) -> Self {
    let effects = self
      .effects
      .into_iter()
      .map(|e| e.security_check())
      .collect();
    Self { effects, ..self }
  }
}
