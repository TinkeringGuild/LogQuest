use super::{
  GINAEarlyEnder, GINATimerStartBehavior, GINATimerTrigger, GINATimerType, GINATrigger,
  GINATriggerGroup, GINATriggers,
};
use crate::common::duration::Duration;
use crate::common::progress_reporter::ProgressReporter;
use crate::common::timestamp::Timestamp;
use crate::common::{maybe_blank, random_id, UUID};
use crate::matchers;
use crate::triggers::effects::Effect;
use crate::triggers::template_string::TemplateString;
use crate::triggers::timers::{Stopwatch, Timer, TimerEffect, TimerStartPolicy, TimerTag};
use crate::triggers::{Trigger, TriggerGroup, TriggerGroupDescendant};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum GINAConversionError {
  #[error("Could not convert the pattern for trigger named `{0}`")]
  TriggerPatternError(String),
  #[error("Encountered unknown Timer duration for timer with name `{0:?}`")]
  TimerDurationError(TemplateString),
  #[error("Encountered unknown Timer Start Behavior for timer with name `{0:?}`")]
  TimerStartPolicyError(TemplateString),
  #[error("Encountered an invalid Early Ender")]
  EarlyEnderPatternError,
  #[error("Invalid Regex in the GINA file")]
  RegexError(#[from] fancy_regex::Error),
}

impl GINATriggers {
  pub fn to_lq(
    &self,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<Vec<TriggerGroup>, GINAConversionError> {
    let mut trigger_groups = Vec::with_capacity(self.trigger_groups.len());
    for tg in self.trigger_groups.iter() {
      trigger_groups.push(tg.to_lq(&import_time, progress)?);
    }
    Ok(trigger_groups)
  }
}

impl GINATriggerGroup {
  fn to_lq(
    &self,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<TriggerGroup, GINAConversionError> {
    progress.update(format!(
      "Converting Trigger Group\n{}",
      maybe_blank(&self.name)
    ));
    // Assume enable_by_default is a shallow-enable, affecting only immediate descendants
    let enable_children = self.enable_by_default.unwrap_or(false);

    let mut children: Vec<TriggerGroupDescendant> =
      Vec::with_capacity(self.trigger_groups.len() + self.triggers.len());

    // Assume TriggerGroups should be first in descendants list
    for tg in self.trigger_groups.iter() {
      children.push(TriggerGroupDescendant::TG(
        tg.to_lq(import_time, &progress)?,
      ));
    }
    for t in self.triggers.iter() {
      let mut trigger = t.to_lq(import_time, progress)?;
      if enable_children {
        trigger.enabled = true;
      }
      children.push(TriggerGroupDescendant::T(trigger));
    }

    Ok(TriggerGroup {
      id: UUID::new(),
      name: self
        .name
        .clone()
        .unwrap_or_else(|| untitled("Trigger Group")),
      comment: self.comments.clone(),
      created_at: import_time.to_owned(),
      updated_at: import_time.to_owned(),
      children,
    })
  }
}

impl GINATrigger {
  /// Converts this GINATrigger to a LogQuest Trigger
  fn to_lq(
    &self,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<Trigger, GINAConversionError> {
    let trigger_id = UUID::new();
    let trigger_name = self.name.clone().unwrap_or_else(|| untitled("Trigger"));
    progress.update(format!("Converting Trigger\n{trigger_name}"));
    Ok(Trigger {
      id: trigger_id.clone(),
      name: trigger_name.clone(),
      comment: self.comments.clone(),
      enabled: true,
      created_at: import_time.to_owned(),
      updated_at: match self.modified {
        Some(naive_datetime) => naive_datetime.into(),
        None => import_time.to_owned(),
      },
      filter: match (self.trigger_text.as_deref(), self.enable_regex) {
        (Some(""), _) => {
          return Err(GINAConversionError::TriggerPatternError(
            trigger_name.to_owned(),
          ))
        }
        (Some(text), Some(true)) => vec![matchers::Matcher::gina(text)?].into(),
        (Some(text), Some(false)) | (Some(text), None) => {
          vec![matchers::Matcher::WholeLine(text.to_owned())].into()
        }
        _ => {
          return Err(GINAConversionError::TriggerPatternError(
            trigger_name.to_owned(),
          ))
        }
      },
      effects: {
        let timer_name: TemplateString = self
          .timer_name
          .clone()
          .unwrap_or_else(|| untitled("Timer"))
          .into();

        let display_text: Option<Effect> = {
          match (&self.use_text, self.display_text.as_deref()) {
            (Some(true), Some("")) => None,
            (Some(true), Some(text)) => Some(Effect::OverlayMessage(text.into())),
            _ => None,
          }
        };

        let copy_text: Option<Effect> = {
          match (&self.copy_to_clipboard, self.clipboard_text.as_deref()) {
            (Some(true), Some("")) => None,
            (Some(true), Some(text)) => Some(Effect::CopyToClipboard(text.into())),
            _ => None,
          }
        };

        let tts = match (
          &self.use_text_to_voice,
          self.text_to_voice_text.as_deref(),
          &self.interrupt_speech,
        ) {
          (_, Some(""), _) => None,
          (Some(true), Some(text), interrupt) => Some(Effect::Speak {
            tmpl: text.into(),
            interrupt: interrupt.unwrap_or(false),
          }),
          _ => None,
        };

        let play_sound_file: Option<Effect> = match self.play_media_file {
          Some(true) => Some(Effect::PlayAudioFile(None)), // the XML does not include the sound file's filepath
          _ => None,
        };

        let timer: Option<Effect> = match self.timer_type {
          None | Some(GINATimerType::NoTimer) => None,

          Some(GINATimerType::Stopwatch) => {
            let stopwatch = Stopwatch {
              name: timer_name.into(),
              // TODO! THIS SHOULD USE CATEGORIES
              tags: vec![],
              effects: {
                if let Some(terminator) = self.early_enders_to_terminator()? {
                  vec![terminator]
                } else {
                  vec![]
                }
              },
            };
            Some(Effect::StartStopwatch(stopwatch))
          }

          // TODO: ARE THERE ANY OTHER DIFFERENCES WITH REPEATING TIMERS?
          Some(GINATimerType::Timer | GINATimerType::RepeatingTimer) => {
            let timer = Timer {
              trigger_id: trigger_id.clone(),
              name: timer_name.clone(),
              tags: vec![/*
                TODO
              */],
              repeats: self.timer_type == Some(GINATimerType::RepeatingTimer),
              duration: match (self.timer_millisecond_duration, self.timer_duration) {
                // Weirdly, GINA's XML has two redundant elements for duration. Prefer millis first
                (Some(millis), _) => Duration::from_millis(millis),
                (None, Some(secs)) => Duration::from_secs(secs),
                _ => return Err(GINAConversionError::TimerDurationError(timer_name)),
              },
              start_policy: match (
                &self.timer_start_behavior,
                &self.restart_based_on_timer_name,
              ) {
                (None, _) => TimerStartPolicy::AlwaysStartNewTimer,
                (Some(GINATimerStartBehavior::IgnoreIfRunning), _) => {
                  TimerStartPolicy::DoNothingIfTimerRunning
                }
                (Some(GINATimerStartBehavior::StartNewTimer), Some(true)) => {
                  TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(
                    timer_name.clone(),
                  )
                }
                (Some(GINATimerStartBehavior::StartNewTimer), Some(false) | None) => {
                  TimerStartPolicy::AlwaysStartNewTimer
                }
                (Some(GINATimerStartBehavior::RestartTimer), Some(true)) => {
                  error!("Encountered unexpected TimerStartBehavior=RestartTimer with RestartBasedOnTimerName=True");
                  return Err(GINAConversionError::TimerStartPolicyError(
                    timer_name.clone(),
                  ));
                }
                (Some(GINATimerStartBehavior::RestartTimer), _) => {
                  TimerStartPolicy::StartAndReplacesAllTimersOfTrigger
                }
              },
              effects: {
                let mut effects: Vec<Effect> = Vec::new();

                // Early Enders with WaitUntilFilterMatches + ClearTimer
                if let Some(terminator) = self.early_enders_to_terminator()? {
                  effects.push(terminator);
                }

                // Timer Ending with WaitUntilSecondsRemain and Parallel effects
                if let Some(secs) = self.timer_ending_time {
                  if secs > 0 {
                    let mut seq = vec![
                      TimerEffect::WaitUntilSecondsRemain(secs).into(),
                      TimerEffect::AddTag(TimerTag::ending()).into(),
                    ];

                    if let (Some(true), Some(ending)) =
                      (self.use_timer_ending, &self.timer_ending_trigger)
                    {
                      if let Some(singularized) =
                        singularize_effects(ending.to_lq(), Effect::Parallel)
                      {
                        seq.push(singularized);
                      }
                    }
                    effects.push(Effect::Sequence(seq));
                  }
                }

                // Timer Ended with WaitUntilFinished and Parallel effects
                if let (Some(true), Some(ended)) = (self.use_timer_ended, &self.timer_ended_trigger)
                {
                  if let Some(singularized) = singularize_effects(ended.to_lq(), Effect::Parallel) {
                    effects.push(Effect::Sequence(vec![
                      TimerEffect::WaitUntilFinished.into(),
                      singularized,
                    ]));
                  }
                }

                effects
              },
            };

            Some(Effect::StartTimer(timer))
          }
        };

        vec![display_text, copy_text, tts, play_sound_file, timer]
          .iter()
          .filter_map(|e| e.to_owned())
          .collect()
      },
    })
  }

  fn early_enders_to_terminator(&self) -> Result<Option<Effect>, GINAConversionError> {
    if self.timer_early_enders.is_empty() {
      return Ok(None);
    }
    let mut enders_filter_matchers = Vec::with_capacity(self.timer_early_enders.len());
    for early_ender in self.timer_early_enders.iter() {
      enders_filter_matchers.push(early_ender.to_lq()?);
    }
    let enders_filter: matchers::FilterWithContext = enders_filter_matchers.into();

    let terminator = Effect::Sequence(vec![
      TimerEffect::WaitUntilFilterMatches(enders_filter).into(),
      TimerEffect::ClearTimer.into(),
    ]);

    Ok(Some(terminator))
  }
}

impl GINATimerTrigger {
  fn to_lq(&self) -> Vec<Effect> {
    let mut timer_effects: Vec<Effect> = vec![];

    match (self.use_text, self.display_text.as_deref()) {
      (Some(true), Some("")) => {}
      (Some(true), Some(text)) => timer_effects.push(Effect::OverlayMessage(text.into())),
      _ => {}
    }

    match (
      self.use_text_to_voice,
      self.text_to_voice_text.as_deref(),
      self.interrupt_speech,
    ) {
      (Some(true), Some(""), _) => {}
      (Some(true), Some(text), interrupt) => {
        timer_effects.push(Effect::Speak {
          tmpl: text.into(),
          interrupt: interrupt.unwrap_or(false),
        });
      }
      _ => {}
    }

    if self.play_media_file.unwrap_or(false) {
      timer_effects.push(Effect::PlayAudioFile(None))
    }

    timer_effects
  }
}

impl GINAEarlyEnder {
  fn to_lq(&self) -> Result<matchers::MatcherWithContext, GINAConversionError> {
    Ok(match (self.enable_regex, &self.early_end_text) {
      (Some(true), Some(pattern)) => matchers::MatcherWithContext::GINA(pattern.to_owned()),
      (Some(false), Some(line)) => matchers::MatcherWithContext::WholeLine(line.to_owned()),
      _ => return Err(GINAConversionError::EarlyEnderPatternError),
    })
  }
}

/// This simplifies the logic when dealing with a vector of Effects which
/// need to be wrapped in a TimerEffect::{Parallel,Sequence} iff there is
/// more than one element in the vector.
fn singularize_effects<E, F>(mut effects: Vec<E>, variant: F) -> Option<E>
where
  F: FnOnce(Vec<E>) -> E,
{
  match effects.as_slice() {
    [] => None,
    [_single] => Some(effects.remove(0)),
    _many => Some(variant(effects)),
  }
}

fn untitled(what: &str) -> String {
  format!("Untitled {} [{}]", what, random_id(4))
}
