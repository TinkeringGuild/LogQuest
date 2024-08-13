use super::{
  GINAEarlyEnder, GINATimerStartBehavior, GINATimerTrigger, GINATimerType, GINATrigger,
  GINATriggerGroup, GINATriggers,
};
use chrono::prelude::*;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};
use zip::read::ZipArchive;

#[derive(thiserror::Error, Debug)]
pub enum GINAParseError {
  #[error("Encountered IO error")]
  IOError(#[from] std::io::Error),
  #[error("Encountered error with ZIP file")]
  ZIPError(#[from] zip::result::ZipError),
  #[error("Encountered error parsing the GINA XML")]
  XMLError(#[from] ::xml::reader::Error),
  #[error("The given GINA package file has an unrecognized file extension")]
  InvalidFileExtension,
  #[error("Encountered unexpected data in the GINA XML: {0}")]
  GINADataError(String),
}

pub fn load_gina_triggers_from_file_path(file_path: &Path) -> Result<GINATriggers, GINAParseError> {
  let file_extension = file_path.extension().and_then(OsStr::to_str);
  let shared_data = match file_extension {
    Some("gtp") => {
      let file = File::open(file_path)?;
      let mut archive = ZipArchive::new(file)?;
      let share_data_xml = archive.by_name("ShareData.xml")?;
      let mut reader = BufReader::new(share_data_xml);
      read_xml(&mut reader)?
    }
    Some("xml") => {
      let file = File::open(file_path)?;
      let mut reader = BufReader::new(file);
      read_xml(&mut reader)?
    }
    Some(_ext) => return Err(GINAParseError::InvalidFileExtension),
    None => return Err(GINAParseError::InvalidFileExtension),
  };
  Ok(shared_data)
}

fn read_xml(reader: impl Read) -> Result<GINATriggers, GINAParseError> {
  let mut parser = EventReader::new(reader);
  let mut shared_data = GINATriggers::new();

  #[allow(unused_assignments)] // the String::new() default value is discarded
  let mut current_element: String = String::new();

  loop {
    match parser.next() {
      Ok(XmlEvent::StartElement { name, .. }) => {
        current_element = name.local_name;
        if current_element == "TriggerGroup" {
          let trigger_group = parse_trigger_group(&mut parser)?;
          shared_data.trigger_groups.push(trigger_group);
        }
      }
      Ok(XmlEvent::EndDocument) => break,
      Err(e) => return Err(GINAParseError::XMLError(e)),
      _ => {}
    }
  }

  Ok(shared_data)
}

fn parse_trigger_group<R: std::io::Read>(
  parser: &mut EventReader<R>,
) -> Result<GINATriggerGroup, GINAParseError> {
  let mut trigger_group = GINATriggerGroup::new();
  let mut current_element = String::new();

  loop {
    match parser.next() {
      Ok(XmlEvent::StartElement { name, .. }) => {
        current_element = name.local_name;
        if current_element == "TriggerGroup" {
          let nested_group = parse_trigger_group(parser)?;
          trigger_group.trigger_groups.push(nested_group);
        } else if current_element == "Trigger" {
          let trigger = parse_trigger(parser)?;
          trigger_group.triggers.push(trigger);
        }
      }
      Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
        "Name" => trigger_group.name = Some(data),
        "Comments" => trigger_group.comments = Some(data),
        "SelfCommented" => trigger_group.self_commented = parse_bool(data),
        "GroupId" => trigger_group.group_id = parse_int(data),
        "EnableByDefault" => trigger_group.enable_by_default = parse_bool(data),
        _ => {}
      },
      Ok(XmlEvent::EndElement { name }) => {
        if name.local_name == "TriggerGroup" {
          break;
        }
      }
      Err(e) => return Err(GINAParseError::XMLError(e)),
      _ => {}
    }
  }

  Ok(trigger_group)
}

fn parse_trigger<R: std::io::Read>(
  parser: &mut EventReader<R>,
) -> Result<GINATrigger, GINAParseError> {
  let mut trigger = GINATrigger::new();
  let mut current_element = String::new();

  loop {
    match parser.next() {
      Ok(XmlEvent::StartElement { name, .. }) => {
        current_element = name.local_name;
        if current_element == "TimerEndingTrigger" {
          trigger.timer_ending_trigger = Some(parse_timer_trigger(parser)?);
        } else if current_element == "TimerEndedTrigger" {
          trigger.timer_ended_trigger = Some(parse_timer_trigger(parser)?);
        } else if current_element == "EarlyEnder" {
          trigger.timer_early_enders.push(parse_early_ender(parser)?);
        }
      }
      Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
        "Name" => trigger.name = Some(data),
        "TriggerText" => trigger.trigger_text = Some(data),
        "Comments" => trigger.comments = Some(data),
        "EnableRegex" => trigger.enable_regex = parse_bool(data),
        "UseText" => trigger.use_text = parse_bool(data),
        "DisplayText" => trigger.display_text = Some(data),
        "CopyToClipboard" => trigger.copy_to_clipboard = parse_bool(data),
        "ClipboardText" => trigger.clipboard_text = Some(data),
        "UseTextToVoice" => trigger.use_text_to_voice = parse_bool(data),
        "InterruptSpeech" => trigger.interrupt_speech = parse_bool(data),
        "TextToVoiceText" => trigger.text_to_voice_text = Some(data),
        "PlayMediaFile" => trigger.play_media_file = parse_bool(data),
        "TimerType" => {
          trigger.timer_type = Some(match data.as_str() {
            "Timer" => GINATimerType::Timer,
            "NoTimer" => GINATimerType::NoTimer,
            "Stopwatch" => GINATimerType::Stopwatch,
            "RepeatingTimer" => GINATimerType::RepeatingTimer,
            _ => {
              return Err(GINAParseError::GINADataError(format!(
                "Unrecognized TimerType: {data}"
              )))
            }
          })
        }
        "TimerName" => trigger.timer_name = Some(data),
        "RestartBasedOnTimerName" => trigger.restart_based_on_timer_name = parse_bool(data),
        "TimerMillisecondDuration" => trigger.timer_millisecond_duration = parse_int(data),
        "TimerDuration" => trigger.timer_duration = parse_int(data),
        "TimerVisibleDuration" => trigger.timer_visible_duration = parse_int(data),
        "TimerStartBehavior" => {
          trigger.timer_start_behavior = Some(match data.as_str() {
            "StartNewTimer" => GINATimerStartBehavior::StartNewTimer,
            "RestartTimer" => GINATimerStartBehavior::RestartTimer,
            "IgnoreIfRunning" => GINATimerStartBehavior::IgnoreIfRunning,
            _ => {
              return Err(GINAParseError::GINADataError(format!(
                "Unrecognized GINA start behavior: {data}"
              )))
            }
          })
        }
        "TimerEndingTime" => trigger.timer_ending_time = parse_int(data),
        "UseTimerEnding" => trigger.use_timer_ending = parse_bool(data),
        "UseTimerEnded" => trigger.use_timer_ended = parse_bool(data),
        "UseCounterResetTimer" => trigger.use_counter_reset_timer = parse_bool(data),
        "CounterResetDuration" => trigger.counter_reset_duration = parse_int(data),
        "Category" => trigger.category = Some(data),
        "Modified" => trigger.modified = parse_datetime(data),
        "UseFastCheck" => trigger.use_fast_check = parse_bool(data),
        _ => {}
      },
      Ok(XmlEvent::EndElement { name }) => {
        if name.local_name == "Trigger" {
          break;
        }
      }
      Err(e) => return Err(GINAParseError::XMLError(e)),
      _ => {}
    }
  }

  Ok(trigger)
}

fn parse_timer_trigger<R: std::io::Read>(
  parser: &mut EventReader<R>,
) -> Result<GINATimerTrigger, GINAParseError> {
  let mut timer_trigger = GINATimerTrigger::new();
  let mut current_element = String::new();

  loop {
    match parser.next() {
      Ok(XmlEvent::StartElement { name, .. }) => {
        current_element = name.local_name;
      }
      Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
        "UseText" => timer_trigger.use_text = parse_bool(data),
        "DisplayText" => timer_trigger.display_text = Some(data),
        "UseTextToVoice" => timer_trigger.use_text_to_voice = parse_bool(data),
        "InterruptSpeech" => timer_trigger.interrupt_speech = parse_bool(data),
        "TextToVoiceText" => timer_trigger.text_to_voice_text = Some(data),
        "PlayMediaFile" => timer_trigger.play_media_file = parse_bool(data),
        _ => {}
      },
      Ok(XmlEvent::EndElement { name }) => {
        if name.local_name == "TimerEndingTrigger" || name.local_name == "TimerEndedTrigger" {
          break;
        }
      }
      Err(e) => return Err(GINAParseError::XMLError(e)),
      _ => {}
    }
  }

  Ok(timer_trigger)
}

fn parse_early_ender<R: std::io::Read>(
  parser: &mut EventReader<R>,
) -> Result<GINAEarlyEnder, GINAParseError> {
  let mut early_ender = GINAEarlyEnder::new();
  let mut current_element = String::new();

  loop {
    match parser.next() {
      Ok(XmlEvent::StartElement { name, .. }) => {
        current_element = name.local_name;
      }
      Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
        "EarlyEndText" => early_ender.early_end_text = Some(data),
        "EnableRegex" => early_ender.enable_regex = parse_bool(data),
        _ => {}
      },
      Ok(XmlEvent::EndElement { name }) => {
        if name.local_name == "EarlyEnder" {
          break;
        }
      }
      Err(e) => return Err(GINAParseError::XMLError(e)),
      _ => {}
    }
  }

  Ok(early_ender)
}

fn parse_int(text: String) -> Option<u32> {
  text.parse().ok()
}

fn parse_bool(text: String) -> Option<bool> {
  match text.as_str() {
    "True" | "true" => Some(true),
    "False" | "false" => Some(false),
    _ => None,
  }
}

fn parse_datetime(date_str: String) -> Option<NaiveDateTime> {
  // example GINA timestamp: "2024-04-10T22:48:35"
  let format = "%Y-%m-%dT%H:%M:%S";
  NaiveDateTime::parse_from_str(&date_str, format).ok()
}
