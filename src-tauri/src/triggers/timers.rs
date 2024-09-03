use super::{effects::EffectWithID, template_string::TemplateString};
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

  pub effects: Vec<EffectWithID>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
pub enum TimerEffect {
  ClearTimer,
  HideTimer,
  RestartTimer,
  UnhideTimer,
  WaitUntilFilterMatches(FilterWithContext, Option<Duration>),
  WaitUntilFinished,
  WaitUntilSecondsRemain(u32),
  // WaitUntilRestarted,
  AddTag(TimerTag),
  RemoveTag(TimerTag),
  WaitUntilTagged(TimerTag),
  IncrementCounter,
  DecrementCounter,
  ResetCounter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub enum TimerStartPolicy {
  AlwaysStartNewTimer,
  DoNothingIfTimerRunning,
  StartAndReplacesAllTimersOfTrigger,
  /// lol, yeah, this has a long name but the verbosity helps me remember what it does
  StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(TemplateString),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub struct Stopwatch {
  pub name: TemplateString,
  pub tags: Vec<TimerTag>,
  pub effects: Vec<EffectWithID>,
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
