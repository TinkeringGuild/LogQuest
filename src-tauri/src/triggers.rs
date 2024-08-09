use crate::{
  common::{
    duration::Duration, timestamp::Timestamp, LogQuestVersionType, LOG_QUEST_VERSION, UUID,
  },
  gina::GINAImport,
  matchers,
  state::config::LogQuestConfig,
};
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};
use tracing::debug;

lazy_static::lazy_static! {

  static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*C\s*\}").unwrap();

  // /// This matches strings that have vars in the form of ${}
  // static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*([\w_]+)\s*\}").unwrap();
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggerRoot {
  groups: Vec<TriggerGroup>,
  log_quest_version: LogQuestVersionType,
}

impl TriggerRoot {
  pub fn new(groups: Vec<TriggerGroup>) -> Self {
    Self {
      groups,
      log_quest_version: LOG_QUEST_VERSION.clone(),
    }
  }

  pub fn ingest_gina_import(&mut self, mut import: GINAImport) {
    self.groups.append(&mut import.converted)
  }

  pub fn iter(&self) -> std::slice::Iter<TriggerGroup> {
    self.groups.iter()
  }

  pub fn trigger_count(&self) -> usize {
    fn descend_descendants(descendants: &Vec<TriggerGroupDescendant>) -> usize {
      descendants.iter().fold(0, |sum, tgd| match tgd {
        TriggerGroupDescendant::T(_) => sum + 1,
        TriggerGroupDescendant::TG(tg) => sum + descend_descendants(&tg.children),
      })
    }
    self
      .groups
      .iter()
      .fold(0, |sum, tg| sum + descend_descendants(&tg.children))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerEffect {
  Parallel(Vec<TriggerEffect>),
  Sequence(Vec<TriggerEffect>),
  /// This uses an Option<String> because importing from GINA does not include
  /// a reference to the sound file, but the TriggerEffect should be preserved when
  /// importing to allow the user to select a file during/after import.
  PlayAudioFile(Option<TemplateString>),
  CopyToClipboard(TemplateString),
  OverlayMessage(TemplateString),
  TextToSpeech(TemplateString),
  StartTimer {
    timer: Timer,
    policy: TimerStartPolicy,
  },
  StartStopwatch(Stopwatch),
  RunSystemCommand(CommandTemplate),

  /// This is meant to be used within a Sequence
  Pause(Duration),
  /// Useful for temporarily disabling an effect or use as a default Effect
  DoNothing,
  // AppendToLog { log_name: String, message: TemplateString }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum TimerEffect {
  Parallel(Vec<TimerEffect>),
  Sequence(Vec<TimerEffect>),
  Pause(Duration),
  PlayAudioFile(Option<TemplateString>),
  DoNothing,
  OverlayMessage(TemplateString),
  Speak(TemplateString),
  SpeakStop,

  HideTimer,
  RestartTimer,
  IncrementCounter,
  DecrementCounter,
  ResetCounter,
  AddTag(TimerTag),
  RemoveTag(TimerTag),
  WaitUntilTagged(TimerTag),
  WaitUntilSecondsRemain(u32),
  WaitUntilFilterMatches(matchers::FilterWithContext),
  // WaitUntilRestarted,
  WaitUntilFinished,
  ClearTimer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TriggerGroup {
  pub id: UUID,
  pub name: String,
  pub comment: Option<String>,
  pub children: Vec<TriggerGroupDescendant>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerGroupDescendant {
  T(Trigger),
  TG(TriggerGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trigger {
  pub id: UUID,
  pub name: String,
  pub comment: Option<String>,
  pub enabled: bool,
  pub filter: matchers::Filter,
  pub effects: Vec<TriggerEffect>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp, // tags: Vec<Tag>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimerStartPolicy {
  AlwaysStartNewTimer,
  DoNothingIfTimerRunning,
  StartAndReplacesAllTimers,
  StartAndReplacesAnyTimerHavingName(String), // TODO: Maybe this should be TemplateString?
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stopwatch {
  pub name: String,
  pub tags: Vec<TimerTag>,
  pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timer {
  pub name: String,
  pub tags: Vec<TimerTag>,
  pub duration: Duration,
  pub timer_start_behavior: TimerStartBehavior,

  /// When finished, the timer starts over until terminated
  pub repeats: bool,

  pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimerStartBehavior {
  StartNewTimer,
  RestartTimer,
  IgnoreIfRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateString {
  tmpl: String,
  param_names: Vec<String>,
}

impl TemplateString {
  pub fn render(&self, char_name: &str) -> String {
    TEMPLATE_VARS.replace_all(&self.tmpl, char_name).to_string()
  }
}

impl From<&str> for TemplateString {
  fn from(tmpl: &str) -> Self {
    let param_names: Vec<String> = TEMPLATE_VARS
      .captures_iter(tmpl)
      // fancy_regex wraps Captures in a Result; TODO: how should this error case be handled?
      .filter_map(|c| c.ok())
      .filter_map(|captures| captures.get(1))
      .map(|mtch| mtch.as_str().to_owned())
      .collect();
    TemplateString {
      tmpl: tmpl.to_owned(),
      param_names,
    }
  }
}
impl From<String> for TemplateString {
  fn from(tmpl: String) -> Self {
    tmpl.as_str().into()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandTemplate {
  pub command: TemplateString,
  pub params: Vec<TemplateString>,
  pub write_to_stdin: Option<TemplateString>,
}

impl From<TriggerGroup> for TriggerGroupDescendant {
  fn from(value: TriggerGroup) -> Self {
    TriggerGroupDescendant::TG(value)
  }
}

impl From<Trigger> for TriggerGroupDescendant {
  fn from(value: Trigger) -> Self {
    TriggerGroupDescendant::T(value)
  }
}

pub fn load_or_create_relative_to_config(config: &LogQuestConfig) -> anyhow::Result<TriggerRoot> {
  let triggers_file_path = config.triggers_file_path();

  if triggers_file_path.exists() {
    debug!(
      "Triggers file exists. Loading {}",
      triggers_file_path.display()
    );
    let reader = BufReader::new(File::open(triggers_file_path)?);
    let root: TriggerRoot = serde_json::from_reader(reader)?;
    Ok(root)
  } else {
    let root = default_triggers();
    config.save_triggers(&root)?;
    Ok(root)
  }
}

#[cfg(not(debug_assertions))]
pub fn default_triggers() -> TriggerRoot {
  TriggerRoot::new(vec![])
}

#[cfg(debug_assertions)]
pub fn default_triggers() -> TriggerRoot {
  TriggerRoot::new(vec![crate::debug_only::test_trigger_group()])
}

#[cfg(test)]
mod test {
  use super::{Trigger, TriggerEffect, TriggerGroup};
  use crate::{
    common::{timestamp::Timestamp, UUID},
    matchers::Matcher,
  };

  #[test]
  fn test_serde() {
    let tg_before = simple_sample();
    let raw_json =
      serde_json::to_string_pretty(&tg_before).expect("Could not convert TriggerGroup to JSON");
    let tg_after: TriggerGroup =
      serde_json::from_str(&raw_json).expect("Could not parse TriggerGroup JSON!");
    assert_eq!(tg_before, tg_after);
  }

  fn simple_sample() -> TriggerGroup {
    let now = Timestamp::now();
    let trigger = Trigger {
      id: UUID::new(),
      name: "Simple Sample Trigger 1".into(),
      enabled: true,
      comment: None,
      created_at: now.clone(),
      updated_at: now.clone(),
      filter: vec![Matcher::gina("^{S1} hits {S2}").unwrap()].into(),
      effects: vec![TriggerEffect::Sequence(vec![
        TriggerEffect::TextToSpeech("This is only a test.".into()),
        TriggerEffect::PlayAudioFile(Some("/dev/null".into())),
      ])],
    };
    TriggerGroup {
      id: UUID::new(),
      name: "Simple Sample Trigger Group".into(),
      comment: Some("I am a comment".into()),
      created_at: now.clone(),
      updated_at: now,
      children: vec![trigger.into()],
    }
  }
}
