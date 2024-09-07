mod conversion;
pub mod importer;
pub mod regex;
pub mod xml;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GINATriggers {
  trigger_groups: Vec<GINATriggerGroup>,
}

impl GINATriggers {
  fn new() -> Self {
    GINATriggers {
      trigger_groups: Vec::new(),
    }
  }
}

#[allow(unused)]
#[derive(Debug, Default, Serialize, Deserialize)]
struct GINATriggerGroup {
  name: Option<String>,
  comments: Option<String>,
  enable_by_default: Option<bool>,
  trigger_groups: Vec<GINATriggerGroup>,
  triggers: Vec<GINATrigger>,

  /// This is ignored during import
  self_commented: Option<bool>,

  // TODO: should this be ignored during import??
  group_id: Option<u32>,
}

impl GINATriggerGroup {
  fn new() -> Self {
    GINATriggerGroup {
      name: None,
      comments: None,
      self_commented: None,
      group_id: None,
      enable_by_default: None,
      trigger_groups: Vec::new(),
      triggers: Vec::new(),
    }
  }
}

#[allow(unused)]
#[derive(Debug, Default, Serialize, Deserialize)]
struct GINATrigger {
  name: Option<String>,
  trigger_text: Option<String>,
  comments: Option<String>,
  category: Option<String>,
  enable_regex: Option<bool>,
  use_text: Option<bool>,
  display_text: Option<String>,
  copy_to_clipboard: Option<bool>,
  clipboard_text: Option<String>,
  use_text_to_voice: Option<bool>,
  interrupt_speech: Option<bool>,
  text_to_voice_text: Option<String>,
  play_media_file: Option<bool>,
  timer_type: Option<GINATimerType>,
  timer_name: Option<String>,
  restart_based_on_timer_name: Option<bool>,
  timer_millisecond_duration: Option<u32>,
  timer_duration: Option<u32>,
  timer_visible_duration: Option<u32>,
  timer_start_behavior: Option<GINATimerStartBehavior>,
  timer_ending_time: Option<u32>,
  use_timer_ending: Option<bool>,
  use_timer_ended: Option<bool>,
  timer_ending_trigger: Option<GINATimerTrigger>,
  timer_ended_trigger: Option<GINATimerTrigger>,
  use_counter_reset_timer: Option<bool>,
  counter_reset_duration: Option<u32>,
  modified: Option<chrono::NaiveDateTime>,
  timer_early_enders: Vec<GINAEarlyEnder>,

  /// This is ignored during import
  use_fast_check: Option<bool>,
}

impl GINATrigger {
  fn new() -> Self {
    GINATrigger {
      name: None,
      trigger_text: None,
      comments: None,
      enable_regex: None,
      use_text: None,
      display_text: None,
      copy_to_clipboard: None,
      clipboard_text: None,
      use_text_to_voice: None,
      interrupt_speech: None,
      text_to_voice_text: None,
      play_media_file: None,
      timer_type: None,
      timer_name: None,
      restart_based_on_timer_name: None,
      timer_millisecond_duration: None,
      timer_duration: None,
      timer_visible_duration: None,
      timer_start_behavior: None,
      timer_ending_time: None,
      use_timer_ending: None,
      use_timer_ended: None,
      timer_ending_trigger: None,
      timer_ended_trigger: None,
      use_counter_reset_timer: None,
      counter_reset_duration: None,
      category: None,
      modified: None,
      use_fast_check: None,
      timer_early_enders: Vec::new(),
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum GINATimerType {
  Timer,
  NoTimer,
  RepeatingTimer,
  Stopwatch,
}

#[derive(Debug, Serialize, Deserialize)]
enum GINATimerStartBehavior {
  StartNewTimer,
  RestartTimer,
  IgnoreIfRunning,
}

/// Used for both <TimerEndingTrigger> and <TimerEndedTrigger>
#[allow(unused)]
#[derive(Debug, Default, Serialize, Deserialize)]
struct GINATimerTrigger {
  /// This means "use display text"
  use_text: Option<bool>,
  display_text: Option<String>,
  use_text_to_voice: Option<bool>,
  interrupt_speech: Option<bool>,
  text_to_voice_text: Option<String>,
  play_media_file: Option<bool>,
}

impl GINATimerTrigger {
  fn new() -> Self {
    GINATimerTrigger {
      use_text: None,
      display_text: None,
      use_text_to_voice: None,
      interrupt_speech: None,
      text_to_voice_text: None,
      play_media_file: None,
    }
  }
}

#[allow(unused)]
#[derive(Debug, Default, Serialize, Deserialize)]
struct GINAEarlyEnder {
  early_end_text: Option<String>,
  enable_regex: Option<bool>,
}

impl GINAEarlyEnder {
  fn new() -> Self {
    GINAEarlyEnder {
      early_end_text: None,
      enable_regex: None,
    }
  }
}
