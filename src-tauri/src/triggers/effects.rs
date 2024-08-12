use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{CommandTemplate, Stopwatch, TemplateString, Timer, TimerStartPolicy, TimerTag};
use crate::{common::duration::Duration, matchers};

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(TS, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
