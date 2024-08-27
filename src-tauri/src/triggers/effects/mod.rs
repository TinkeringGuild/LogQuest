mod clipboard;
mod nothing;
mod overlay_message;
mod parallel;
mod pause;
mod play_audio;
mod sequence;
mod speak;
mod start_timer;
mod sys_cmd;
mod timer_effects;

use super::command_template::{CommandTemplate, CommandTemplateSecurityCheck};
use super::timers::{Stopwatch, Timer, TimerEffect};
use super::TemplateString;
use crate::audio::PlayAudioFileError;
use crate::state::timer_manager::TimerCommand;
use crate::{common::duration::Duration, reactor::EventContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::error;

use clipboard::CopyToClipboardEffect;
use nothing::DoNothingEffect;
use overlay_message::OverlayMessageEffect;
use parallel::EffectParallel;
use pause::PauseEffect;
use play_audio::PlayAudioFileEffect;
use sequence::EffectSequence;
use speak::{SpeakEffect, SpeakStopEffect};
use start_timer::StartTimerEffect;
use sys_cmd::SystemCommandEffect;
use timer_effects::clear::ClearTimerEffect;
use timer_effects::hide::{HideTimerEffect, UnhideTimerEffect};
use timer_effects::restart::RestartTimerEffect;
use timer_effects::wait_until_filter_matches::WaitUntilFilterMatchesTimerEffect;
use timer_effects::wait_until_finished::WaitUntilFinishedEffect;
use timer_effects::wait_until_seconds_remain::WaitUntilSecondsRemainEffect;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
pub enum Effect {
  Parallel(Vec<Effect>),
  Sequence(Vec<Effect>),
  /// This uses an Option<String> because importing from GINA does not include
  /// a reference to the sound file, but the TriggerEffect should be preserved when
  /// importing to allow the user to select a file during/after import.
  PlayAudioFile(Option<TemplateString>),
  CopyToClipboard(TemplateString),
  OverlayMessage(TemplateString),
  StartTimer(Timer),
  StartStopwatch(Stopwatch),
  RunSystemCommand(CommandTemplateSecurityCheck),
  SpeakStop,
  Speak {
    tmpl: TemplateString,
    interrupt: bool,
  },

  /// This is only valid for use in a Timer `effects` field
  ScopedTimerEffect(TimerEffect),

  /// This is meant to be used within a Sequence
  Pause(Duration),
  /// Useful for temporarily disabling an effect or use as a default Effect
  DoNothing,
  // AppendToLog { log_name: String, message: TemplateString }
}

#[derive(thiserror::Error, Debug)]
pub enum EffectError {
  #[error("Multiple EffectErrors occurred")]
  Multiple(Vec<EffectError>),

  #[error(transparent)]
  AudioError(#[from] PlayAudioFileError),

  #[error(transparent)]
  TauriError(#[from] tauri::Error),

  #[error(transparent)]
  CommandIOError(#[from] std::io::Error),

  #[error("Command `{0}` failed with status code {1}")]
  CommandFailure(String, i32),

  #[error("Command `{0}` CRASHED (with no status code)")]
  CommandDied(String),

  #[error("Failed to execute command: {0}")]
  CommandStdinClosedError(String),

  #[error("Refused to execute an unapproved CommandTemplate: {0:?}")]
  CommandSecurityCheckFail(CommandTemplate),

  #[error("Encountered a TimerEffect created incorrectly! No TimerContext")]
  TimerEffectWithoutTimerContext,

  #[error("Tried to send a message to a dead Timer")]
  TimerCommandChannelClosedError(#[from] mpsc::error::SendError<TimerCommand>),
}

pub type EffectResult = Result<(), EffectError>;

#[async_trait]
pub trait ReadyEffect
where
  Self: Send,
{
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult;
}

impl Effect {
  pub fn ready(self) -> Box<dyn ReadyEffect> {
    match self {
      Self::DoNothing => Box::new(DoNothingEffect),
      Self::Parallel(effects) => {
        let effects = effects.into_iter().map(|e| e.ready()).collect();
        Box::new(EffectParallel(effects))
      }
      Self::Sequence(effects) => {
        let effects = effects.into_iter().map(|e| e.ready()).collect();
        Box::new(EffectSequence(effects))
      }
      Self::StartTimer(timer) => Box::new(StartTimerEffect(timer)),
      Self::OverlayMessage(tmpl) => Box::new(OverlayMessageEffect(tmpl)),
      Self::CopyToClipboard(tmpl) => Box::new(CopyToClipboardEffect(tmpl)),
      Self::PlayAudioFile(None) => Box::new(DoNothingEffect),
      Self::PlayAudioFile(Some(tmpl)) => Box::new(PlayAudioFileEffect(tmpl)),
      Self::SpeakStop => Box::new(SpeakStopEffect),
      Self::Speak { tmpl, interrupt } => Box::new(SpeakEffect {
        tmpl,
        interrupt,
        non_blocking: false, // TODO: Expose this as an option to the end-user
      }),
      Self::Pause(duration) => Box::new(PauseEffect(duration)),
      Self::RunSystemCommand(cmd_tmpl_sec_check) => {
        match cmd_tmpl_sec_check.security_check() {
          cmd_tmpl_sec_check @ CommandTemplateSecurityCheck::Approved(_, _) => {
            Box::new(SystemCommandEffect {
              cmd_tmpl_sec_check,
              non_blocking: false, // TODO: Expose this as an option to the end-user
            })
          }
          CommandTemplateSecurityCheck::Unapproved(cmd_tmpl) => {
            error!("Refusing to execute unapproved CommandTemplate: {cmd_tmpl:?}");
            Box::new(DoNothingEffect)
          }
        }
      }
      Self::ScopedTimerEffect(timer_effect) => match timer_effect {
        TimerEffect::WaitUntilFilterMatches(filter, duration) => {
          Box::new(WaitUntilFilterMatchesTimerEffect(filter, duration))
        }
        TimerEffect::HideTimer => Box::new(HideTimerEffect),
        TimerEffect::UnhideTimer => Box::new(UnhideTimerEffect),
        TimerEffect::RestartTimer => Box::new(RestartTimerEffect),
        TimerEffect::WaitUntilFinished => Box::new(WaitUntilFinishedEffect),
        TimerEffect::ClearTimer => Box::new(ClearTimerEffect),
        TimerEffect::WaitUntilSecondsRemain(secs) => Box::new(WaitUntilSecondsRemainEffect(secs)),

        TimerEffect::IncrementCounter
        | TimerEffect::DecrementCounter
        | TimerEffect::ResetCounter
        | TimerEffect::AddTag(_)
        | TimerEffect::RemoveTag(_)
        | TimerEffect::WaitUntilTagged(_) => {
          error!("UNIMPLEMENTED TIMER EFFECT: {timer_effect:?}");
          Box::new(DoNothingEffect)
        }
      },
      Self::StartStopwatch(_) => {
        // TODO!
        Box::new(DoNothingEffect)
      }
    }
  }

  pub(super) fn security_check(self) -> Self {
    match self {
      Self::RunSystemCommand(cmd_tmpl_sec_check) => {
        Self::RunSystemCommand(cmd_tmpl_sec_check.security_check())
      }
      Self::StartTimer(timer) => Self::StartTimer(timer.security_check()),
      Self::StartStopwatch(stopwatch) => Self::StartStopwatch(stopwatch.security_check()),
      Self::Parallel(effects) => {
        Self::Parallel(effects.into_iter().map(|e| e.security_check()).collect())
      }
      Self::Sequence(effects) => {
        Self::Sequence(effects.into_iter().map(|e| e.security_check()).collect())
      }
      other => other,
    }
  }
}

impl From<TimerEffect> for Effect {
  fn from(timer_effect: TimerEffect) -> Self {
    Effect::ScopedTimerEffect(timer_effect)
  }
}
