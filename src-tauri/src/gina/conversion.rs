use super::{
  GINAEarlyEnder, GINATimerStartBehavior, GINATimerTrigger, GINATimerType, GINATrigger,
  GINATriggerGroup, GINATriggers,
};
use crate::common::duration::Duration;
use crate::common::progress_reporter::ProgressReporter;
use crate::common::timestamp::Timestamp;
use crate::common::{maybe_blank, random_id, UUID};
use crate::matchers;
use crate::triggers::effects::{Effect, EffectWithID};
use crate::triggers::template_string::TemplateString;
use crate::triggers::timers::{Stopwatch, Timer, TimerEffect, TimerStartPolicy, TimerTag};
use crate::triggers::trigger_index::{
  DataMutationError, Mutation, TriggerGroupDescendant, TriggerIndex, TriggerTag,
};
use crate::triggers::{Trigger, TriggerGroup};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum GINAImportError {
  #[error(transparent)]
  MutationError(#[from] DataMutationError),
  #[error(transparent)]
  ConversionError(#[from] GINAConversionError),
}

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
  pub fn convert_import(
    self,
    index: &mut TriggerIndex,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<(), GINAImportError> {
    let category_tags: HashMap<String, UUID> = self
      .get_distinct_categories()
      .into_iter()
      .map(|category| {
        let TriggerTag { id, .. } = index.create_trigger_tag(&category);
        (category.clone(), id)
      })
      .collect();

    for gina_trigger_group in self.trigger_groups.into_iter() {
      gina_trigger_group.convert_import(index, None, &category_tags, &import_time, progress)?;
    }
    Ok(())
  }

  fn get_distinct_categories(&self) -> HashSet<String> {
    let mut categories = HashSet::new();
    let mut queue: VecDeque<&GINATriggerGroup> = self.trigger_groups.iter().collect();
    while let Some(gina_group) = queue.pop_front() {
      queue.extend(gina_group.trigger_groups.iter());
      for trigger in gina_group.triggers.iter() {
        if let Some(category) = &trigger.category {
          categories.insert(category.clone());
        }
      }
    }
    categories
  }
}

impl GINATriggerGroup {
  fn convert_import(
    self,
    index: &mut TriggerIndex,
    parent_id: Option<UUID>,
    category_tags: &HashMap<String, UUID>,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<UUID, GINAImportError> {
    progress.update(format!(
      "Converting Trigger Group\n{}",
      maybe_blank(&self.name)
    ));

    let this_id = UUID::new();

    // Assume enable_by_default is a shallow-enable, affecting only immediate descendants
    let enable_children = self.enable_by_default.unwrap_or(false);

    let mut children: Vec<TriggerGroupDescendant> =
      Vec::with_capacity(self.triggers.len() + self.trigger_groups.len());

    // Assume TriggerGroups should be first in descendants list. The order is CURRENTLY lost in the GINA importer.
    for gina_group in self.trigger_groups.into_iter() {
      let group_id = gina_group.convert_import(
        index,
        Some(this_id.clone()),
        category_tags,
        import_time,
        progress,
      )?;
      children.push(TriggerGroupDescendant::G(group_id));
    }

    for gina_trigger in self.triggers.into_iter() {
      let trigger_id = gina_trigger.convert_import(
        index,
        this_id.clone(),
        category_tags,
        import_time,
        progress,
      )?;

      // // TODO!! I NEED TO AUTO-TAG THE TRIGGERS TO ENABLE THEM
      // if enable_children {
      //   trigger.enabled = true;
      // }

      children.push(TriggerGroupDescendant::T(trigger_id));
    }

    let this = TriggerGroup {
      id: this_id.clone(),
      parent_id,
      name: self
        .name
        .clone()
        .unwrap_or_else(|| untitled("Trigger Group")),
      comment: self.comments.clone(),
      created_at: import_time.to_owned(),
      updated_at: import_time.to_owned(),
      children,
    };

    index.import_trigger_group(this);

    Ok(this_id)
  }
}

impl GINATrigger {
  /// Converts this GINATrigger to a LogQuest Trigger
  fn convert_import(
    self,
    index: &mut TriggerIndex,
    parent_id: UUID, // not an Option<UUID> because we assume all Triggers belong to a group
    category_tags: &HashMap<String, UUID>,
    import_time: &Timestamp,
    progress: &ProgressReporter,
  ) -> Result<UUID, GINAImportError> {
    let trigger_id = UUID::new();
    let trigger_name = self.name.clone().unwrap_or_else(|| untitled("Trigger"));
    progress.update(format!("Converting Trigger\n{trigger_name}"));

    let updated_at = match self.modified {
      Some(naive_datetime) => naive_datetime.into(),
      None => import_time.to_owned(),
    };

    let filter = match (self.trigger_text.as_deref(), self.enable_regex) {
      (Some(""), _) => {
        return Err(GINAConversionError::TriggerPatternError(trigger_name.to_owned()).into())
      }
      (Some(text), Some(true)) => {
        vec![matchers::Matcher::gina(text).map_err(GINAConversionError::from)?].into()
      }
      (Some(text), Some(false)) | (Some(text), None) => {
        vec![matchers::Matcher::WholeLine(text.to_owned())].into()
      }
      _ => return Err(GINAConversionError::TriggerPatternError(trigger_name.to_owned()).into()),
    };

    let effects = {
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
            tags: vec![/* TODO! */],
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
            name_tmpl: timer_name.clone(),
            tags: vec![/* TODO! */],
            repeats: self.timer_type == Some(GINATimerType::RepeatingTimer),
            duration: match (self.timer_millisecond_duration, self.timer_duration) {
              // Weirdly, GINA's XML has two redundant elements for duration. Prefer millis first
              (Some(millis), _) => Duration::from_millis(millis),
              (None, Some(secs)) => Duration::from_secs(secs),
              _ => return Err(GINAConversionError::TimerDurationError(timer_name).into()),
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
                return Err(GINAConversionError::TimerStartPolicyError(timer_name.clone()).into());
              }
              (Some(GINATimerStartBehavior::RestartTimer), _) => {
                TimerStartPolicy::StartAndReplacesAllTimersOfTrigger
              }
            },
            effects: {
              let mut effects: Vec<EffectWithID> = Vec::new();

              // Early Enders with WaitUntilFilterMatches + ClearTimer
              if let Some(terminator) = self.early_enders_to_terminator()? {
                effects.push(terminator);
              }

              // Timer Ending with WaitUntilSecondsRemain and Parallel effects
              if let Some(secs) = self.timer_ending_time {
                if secs > 0 {
                  let mut seq = vec![
                    EffectWithID::new(TimerEffect::WaitUntilSecondsRemain(secs).into()),
                    EffectWithID::new(TimerEffect::AddTag(TimerTag::ending()).into()),
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
                  effects.push(EffectWithID::new(Effect::Sequence(seq)));
                }
              }

              // Timer Ended with WaitUntilFinished and Parallel effects
              if let (Some(true), Some(ended)) = (self.use_timer_ended, &self.timer_ended_trigger) {
                if let Some(singularized) = singularize_effects(ended.to_lq(), Effect::Parallel) {
                  effects.push(EffectWithID::new(Effect::Sequence(vec![
                    EffectWithID::new(TimerEffect::WaitUntilFinished.into()),
                    singularized,
                  ])));
                }
              }

              effects
            },
          };

          Some(Effect::StartTimer(timer))
        }
      };

      vec![display_text, copy_text, tts, play_sound_file, timer]
        .into_iter()
        .filter_map(|e| e)
        .map(EffectWithID::new)
        .collect()
    };

    let this = Trigger {
      id: trigger_id.clone(),
      parent_id: Some(parent_id),
      name: trigger_name,
      comment: self.comments,
      enabled: true,
      created_at: import_time.to_owned(),
      updated_at,
      filter,
      effects,
    };

    index.import_trigger(this);

    if let Some(category) = self.category {
      if let Some(tag_id) = category_tags.get(&category) {
        index.mutate(Mutation::TagTrigger {
          trigger_id: trigger_id.clone(),
          trigger_tag_id: tag_id.clone(),
        })?;
      }
    }

    Ok(trigger_id)
  }

  fn early_enders_to_terminator(&self) -> Result<Option<EffectWithID>, GINAConversionError> {
    if self.timer_early_enders.is_empty() {
      return Ok(None);
    }
    let mut enders_filter_matchers = Vec::with_capacity(self.timer_early_enders.len());
    for early_ender in self.timer_early_enders.iter() {
      enders_filter_matchers.push(early_ender.to_lq()?);
    }
    let enders_filter: matchers::FilterWithContext = enders_filter_matchers.into();

    let terminator = Effect::Sequence(vec![
      EffectWithID::new(TimerEffect::WaitUntilFilterMatches(enders_filter, None).into()),
      EffectWithID::new(TimerEffect::ClearTimer.into()),
    ]);

    Ok(Some(EffectWithID::new(terminator)))
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
fn singularize_effects<F>(mut effects: Vec<Effect>, variant: F) -> Option<EffectWithID>
where
  F: FnOnce(Vec<EffectWithID>) -> Effect,
{
  match effects.as_slice() {
    [] => None,
    [_single] => Some(EffectWithID::new(effects.remove(0))),
    _many => {
      let effects = effects.into_iter().map(EffectWithID::new).collect();
      Some(EffectWithID::new(variant(effects)))
    }
  }
}

fn untitled(what: &str) -> String {
  format!("Untitled {} [{}]", what, random_id(4))
}
