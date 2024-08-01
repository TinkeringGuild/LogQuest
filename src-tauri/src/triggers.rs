use crate::{
  common::{duration::Duration, timestamp::Timestamp},
  gina::regex::CapturesGINA,
  matchers,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {

  static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*C\s*\}").unwrap();

  // /// This matches strings that have vars in the form of ${}
  // static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*([\w_-]+)\s*\}").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerStartPolicy {
  AlwaysStartNewTimer,
  DoNothingIfTimerRunning,
  StartAndReplacesAllTimers,
  StartAndReplacesAnyTimerWithName(String), // maybe this should be TemplateString?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
  WaitUntilFilterMatches(matchers::Filter),
  // WaitUntilRestarted,
  WaitUntilFinished,
  ClearTimer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerGroup {
  pub name: String,
  pub comment: Option<String>,
  pub children: Vec<TriggerGroupDescendant>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerGroupDescendant {
  T(Trigger),
  TG(TriggerGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
  pub name: String,
  pub comment: Option<String>,
  pub enabled: bool,
  pub filter: matchers::Filter,
  pub effects: Vec<TriggerEffect>,
  pub created_at: Timestamp,
  pub updated_at: Timestamp, // tags: Vec<Tag>
}
impl Trigger {
  pub fn captures(&self, line: &str, char_name: &str) -> Option<CapturesGINA> {
    for matcher in self.filter.iter() {
      if let matchers::Matcher::GINA(regex_gina) = matcher {
        if let Some(captures_gina) = regex_gina.check(line, char_name) {
          return Some(captures_gina);
        }
      }
    }
    None
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stopwatch {
  pub name: String,
  pub tags: Vec<TimerTag>,
  pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
  pub name: String,
  pub tags: Vec<TimerTag>,
  pub duration: Duration,
  pub timer_start_behavior: TimerStartBehavior,

  /// When finished, the timer starts over until terminated
  pub repeats: bool,

  pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerStartBehavior {
  StartNewTimer,
  RestartTimer,
  IgnoreIfRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    // TODO: Compile this regex at compilation time with lazy_static or a macro
    let param_names: Vec<String> = TEMPLATE_VARS
      .captures_iter(tmpl)
      .filter_map(|capture| capture.get(1).map(|c| c.as_str().to_owned()))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTemplate {
  pub command: TemplateString,
  pub params: Vec<TemplateString>,
  pub write_to_stdin: Option<TemplateString>,
}
