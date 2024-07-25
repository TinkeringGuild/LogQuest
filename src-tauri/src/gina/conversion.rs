use super::{
  GINAEarlyEnder, GINATimerStartBehavior, GINATimerTrigger, GINATimerType, GINATrigger,
  GINATriggerGroup, GINATriggers,
};
use crate::utils::random_id;
use crate::{matchers, triggers};
use anyhow::bail;
use chrono::prelude::*;

impl GINATriggers {
  pub fn to_lq(&self) -> anyhow::Result<Vec<triggers::TriggerGroup>> {
    let mut trigger_groups = Vec::with_capacity(self.trigger_groups.len());
    for tg in self.trigger_groups.iter() {
      trigger_groups.push(tg.to_lq()?);
    }
    Ok(trigger_groups)
  }
}

impl GINATriggerGroup {
  fn to_lq(&self) -> anyhow::Result<triggers::TriggerGroup> {
    // Assume enable_by_default is a shallow-enable, affecting only immediate descendants
    let enable_children = self.enable_by_default.unwrap_or(false);

    let mut children: Vec<triggers::TriggerGroupDescendant> =
      Vec::with_capacity(self.trigger_groups.len() + self.triggers.len());

    // Assume TriggerGroups should be first in descendants list
    for tg in self.trigger_groups.iter() {
      children.push(triggers::TriggerGroupDescendant::TG(tg.to_lq()?));
    }
    for t in self.triggers.iter() {
      let mut trigger = t.to_lq()?;
      if enable_children {
        trigger.enabled = true;
      }
      children.push(triggers::TriggerGroupDescendant::T(trigger));
    }

    Ok(triggers::TriggerGroup {
      name: self
        .name
        .clone()
        .unwrap_or_else(|| untitled("Trigger Group")),
      comment: self.comments.clone(),
      created_at: Utc::now(),
      children,
    })
  }
}

impl GINATrigger {
  /// Converts this GINATrigger to a LogQuest Trigger
  fn to_lq(&self) -> anyhow::Result<triggers::Trigger> {
    let trigger_name = self.name.clone().unwrap_or_else(|| untitled("Trigger"));
    Ok(triggers::Trigger {
      name: trigger_name.clone(),
      comment: self.comments.clone(),
      enabled: true,
      last_modified: match self.modified {
        Some(naive_datetime) => naive_datetime.and_utc(),
        _ => Utc::now(),
      },
      filter: match (self.trigger_text.as_deref(), self.enable_regex) {
        (Some(""), _) => bail!("GINA trigger {} had no contents", &trigger_name),
        (Some(text), Some(true)) => {
          vec![matchers::Matcher::GINAPattern(text.to_owned())]
        }
        (Some(text), Some(false)) | (Some(text), None) => {
          vec![matchers::Matcher::WholeLine(text.to_owned())]
        }
        _ => bail!("Cannot interpret GINA trigger text for {}", &trigger_name),
      },
      effects: {
        // TODO: render the name of the timer here with TemplateString
        let timer_name = self.timer_name.clone().unwrap_or_else(|| untitled("Timer"));

        let display_text: Option<triggers::TriggerEffect> = effect_from_options(
          &self.use_text,
          &self.display_text,
          triggers::TriggerEffect::OverlayMessage,
        );

        let copy_text: Option<triggers::TriggerEffect> = effect_from_options(
          &self.copy_to_clipboard,
          &self.clipboard_text,
          triggers::TriggerEffect::CopyToClipboard,
        );

        // TODO: This needs to handle self.interrupt_speech
        let tts: Option<triggers::TriggerEffect> = effect_from_options(
          &self.use_text_to_voice,
          &self.text_to_voice_text,
          triggers::TriggerEffect::TextToSpeech,
        );

        let play_sound_file: Option<triggers::TriggerEffect> = match self.play_media_file {
          Some(true) => Some(triggers::TriggerEffect::PlayAudioFile(None)), // the XML does not include the sound file's filepath
          _ => None,
        };

        let timer: Option<triggers::TriggerEffect> = match self.timer_type {
          None | Some(GINATimerType::NoTimer) => None,

          Some(GINATimerType::Stopwatch) => {
            let stopwatch = triggers::Stopwatch {
              name: timer_name.into(),
              // TODO! THIS SHOULD USE CATEGORIES
              tags: vec![],
              updates: {
                if let Some(terminator) = self.early_enders_to_terminator()? {
                  vec![terminator]
                } else {
                  vec![]
                }
              },
            };
            Some(triggers::TriggerEffect::StartStopwatch(stopwatch))
          }

          // TODO: ARE THERE ANY OTHER DIFFERENCES WITH REPEATING TIMERS?
          Some(GINATimerType::Timer | GINATimerType::RepeatingTimer) => {
            let timer = triggers::Timer {
              name: timer_name.clone(),
              // TODO: tags should belong to a GINAImport type.
              tags: vec![],
              repeats: self.timer_type == Some(GINATimerType::RepeatingTimer),
              duration: match (self.timer_millisecond_duration, self.timer_duration) {
                // Weirdly, GINA's XML has two redundant elements for duration. Prefer millis first
                (Some(millis), _) => triggers::Duration::from_millis(millis),
                (None, Some(secs)) => triggers::Duration::from_secs(secs),
                _ => bail!("Could not determine Timer duration for timer {timer_name}!",),
              },
              timer_start_behavior: match &self.timer_start_behavior {
                Some(b) => b.to_lq(),
                None => {
                  bail!("Timer Start Behavior unknown for timer {timer_name}!")
                }
              },
              updates: {
                let mut updates: Vec<triggers::TimerEffect> = Vec::new();

                // Early Enders with WaitUntilFilterMatches + ClearTimer
                if let Some(terminator) = self.early_enders_to_terminator()? {
                  updates.push(terminator);
                }

                // Timer Ending with WaitUntilSecondsRemain and Parallel effects
                if let Some(secs) = self.timer_ending_time {
                  if secs > 0 {
                    let mut seq = vec![
                      triggers::TimerEffect::WaitUntilSecondsRemain(secs),
                      triggers::TimerEffect::AddTag(triggers::TimerTag::ending()),
                    ];

                    if let (Some(true), Some(ending)) =
                      (self.use_timer_ending, &self.timer_ending_trigger)
                    {
                      if let Some(singularized) =
                        singularize_effects(ending.to_lq(), triggers::TimerEffect::Parallel)
                      {
                        seq.push(singularized);
                      }
                    }
                    updates.push(triggers::TimerEffect::Sequence(seq));
                  }
                }

                // Timer Ended with WaitUntilFinished and Parallel effects
                if let (Some(true), Some(ended)) = (self.use_timer_ended, &self.timer_ended_trigger)
                {
                  if let Some(singularized) =
                    singularize_effects(ended.to_lq(), triggers::TimerEffect::Parallel)
                  {
                    updates.push(triggers::TimerEffect::Sequence(vec![
                      triggers::TimerEffect::WaitUntilFinished,
                      singularized,
                    ]));
                  }
                }

                updates
              },
            };

            let policy: triggers::TimerStartPolicy = match (
              &self.timer_start_behavior,
              &self.restart_based_on_timer_name,
            ) {
              (None, _) => triggers::TimerStartPolicy::AlwaysStartNewTimer,
              (Some(GINATimerStartBehavior::IgnoreIfRunning), _) => {
                triggers::TimerStartPolicy::DoNothingIfTimerRunning
              }
              (Some(GINATimerStartBehavior::StartNewTimer), Some(true)) => {
                triggers::TimerStartPolicy::StartAndReplacesAnyTimerWithName(timer_name)
              }
              (Some(GINATimerStartBehavior::StartNewTimer), Some(false) | None) => {
                triggers::TimerStartPolicy::AlwaysStartNewTimer
              }
              (Some(GINATimerStartBehavior::RestartTimer), Some(true)) => {
                bail!("Encountered unexpected TimerStartBehavior=RestartTimer with RestartBasedOnTimerName=True")
              }
              (Some(GINATimerStartBehavior::RestartTimer), _) => {
                triggers::TimerStartPolicy::StartAndReplacesAllTimers
              }
            };
            Some(triggers::TriggerEffect::StartTimer { timer, policy })
          }
        };

        vec![display_text, copy_text, tts, play_sound_file, timer]
          .iter()
          .filter_map(|e| e.to_owned())
          .collect()
      },
    })
  }

  fn early_enders_to_terminator(&self) -> anyhow::Result<Option<triggers::TimerEffect>> {
    if self.timer_early_enders.is_empty() {
      return Ok(None);
    }
    let mut enders_filter: matchers::Filter = Vec::with_capacity(self.timer_early_enders.len());
    for early_ender in self.timer_early_enders.iter() {
      enders_filter.push(early_ender.to_lq()?);
    }

    let terminator = triggers::TimerEffect::Sequence(vec![
      triggers::TimerEffect::WaitUntilFilterMatches(enders_filter),
      triggers::TimerEffect::ClearTimer,
    ]);

    Ok(Some(terminator))
  }
}

impl GINATimerStartBehavior {
  fn to_lq(&self) -> triggers::TimerStartBehavior {
    match self {
      Self::StartNewTimer => triggers::TimerStartBehavior::StartNewTimer,
      Self::RestartTimer => triggers::TimerStartBehavior::RestartTimer,
      Self::IgnoreIfRunning => triggers::TimerStartBehavior::IgnoreIfRunning,
    }
  }
}

impl GINATimerTrigger {
  fn to_lq(&self) -> Vec<triggers::TimerEffect> {
    let mut timer_effects: Vec<triggers::TimerEffect> = vec![];

    match (self.use_text, self.display_text.as_deref()) {
      (Some(true), Some("")) => {}
      (Some(true), Some(text)) => {
        timer_effects.push(triggers::TimerEffect::OverlayMessage(text.into()))
      }
      _ => {}
    }

    match (
      self.use_text_to_voice,
      self.text_to_voice_text.as_deref(),
      self.interrupt_speech,
    ) {
      (Some(true), Some(""), _) => {}
      (Some(true), Some(text), Some(false) | None) => {
        timer_effects.push(triggers::TimerEffect::Speak(text.into()))
      }
      (Some(true), Some(text), Some(true)) => {
        timer_effects.push(triggers::TimerEffect::Sequence(vec![
          triggers::TimerEffect::SpeakStop,
          triggers::TimerEffect::Speak(text.into()),
        ]))
      }
      _ => {}
    }

    if self.play_media_file.unwrap_or(false) {
      timer_effects.push(triggers::TimerEffect::PlayAudioFile(None))
    }

    timer_effects
  }
}

impl GINAEarlyEnder {
  fn to_lq(&self) -> anyhow::Result<matchers::Matcher> {
    Ok(match (self.enable_regex, self.early_end_text.clone()) {
      (Some(true), Some(pattern)) => matchers::Matcher::GINAPattern(pattern),
      (Some(false), Some(line)) => matchers::Matcher::WholeLine(line),
      _ => bail!("Invalid Early Ender"),
    })
  }
}

fn effect_from_options<F>(
  condition: &Option<bool>,
  text: &Option<String>,
  converter: F,
) -> Option<triggers::TriggerEffect>
where
  F: FnOnce(triggers::TemplateString) -> triggers::TriggerEffect,
{
  match (condition, text.as_deref()) {
    (Some(true), Some("")) => None,
    (Some(true), Some(text)) => Some(converter(text.into())),
    _ => None,
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
