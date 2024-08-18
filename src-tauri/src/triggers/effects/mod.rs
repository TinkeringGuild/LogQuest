mod clipboard;
mod parallel;
mod sequence;

use super::{CommandTemplate, Stopwatch, TemplateString, Timer, TimerTag};
use crate::common::duration::Duration;
use crate::matchers;
use crate::matchers::MatchContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum EffectError {
  #[error("Multiple EffectErrors occurred")]
  Multiple(Vec<EffectError>),
  #[error(transparent)]
  TauriError(#[from] tauri::Error),
}

pub type EffectResult = Result<(), EffectError>;

#[async_trait]
pub trait ReadyEffect
where
  Self: Send,
{
  async fn fire(self: Box<Self>) -> EffectResult;
}

pub trait EffectTemplate {
  fn ready(&self, context: &MatchContext) -> Box<dyn ReadyEffect>;
}

////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////
////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
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
  StartTimer(Timer),
  StartStopwatch(Stopwatch),
  RunSystemCommand(CommandTemplate),

  /// This is meant to be used within a Sequence
  Pause(Duration),
  /// Useful for temporarily disabling an effect or use as a default Effect
  DoNothing,
  // AppendToLog { log_name: String, message: TemplateString }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
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
