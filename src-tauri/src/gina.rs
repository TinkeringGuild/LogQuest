use anyhow::bail;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xml::reader::XmlEvent;
use zip::read::ZipArchive;

pub fn load_gina_triggers_from_file_path(file_path: PathBuf) -> anyhow::Result<GINATriggers> {
    let shared_data = match file_path.extension().and_then(|s| s.to_str()) {
        Some("gtp") => {
            let file = File::open(&file_path)?;
            let mut archive = ZipArchive::new(file)?;
            let share_data_xml = archive.by_name("ShareData.xml").map_err(|_| {
                anyhow::anyhow!("Could not find a ShareData.xml file in the GTP archive")
            })?;
            let mut reader = BufReader::new(share_data_xml);
            read_xml(&mut reader)?
        }
        Some("xml") => {
            let file = File::open(&file_path)?;
            let mut reader = BufReader::new(file);
            read_xml(&mut reader)?
        }
        Some(ext) => {
            bail!("Unrecognized GINA trigger file format: {}", ext);
        }
        None => {
            bail!("GINA trigger file must have a .gtp or .xml extension");
        }
    };
    Ok(shared_data)
}

fn read_xml(reader: impl Read) -> anyhow::Result<GINATriggers> {
    let mut parser = xml::reader::EventReader::new(reader);
    let mut shared_data = GINATriggers::new();

    #[allow(unused_assignments)] // the String::new() default value is discarded
    let mut current_element: String = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name;
                if current_element == "TriggerGroup" {
                    let trigger_group = parse_trigger_group(&mut parser);
                    shared_data.trigger_groups.push(trigger_group);
                }
            }
            Ok(XmlEvent::EndDocument) => break,
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(shared_data)
}

#[allow(unused)]
#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
struct GINATriggerGroup {
    name: Option<String>,
    comments: Option<String>,
    self_commented: Option<bool>,
    group_id: Option<i32>,
    enable_by_default: Option<bool>,
    trigger_groups: Vec<GINATriggerGroup>,
    triggers: Vec<GINATrigger>,
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
#[derive(Debug, Default)]
struct GINATrigger {
    name: Option<String>,
    trigger_text: Option<String>,
    comments: Option<String>,
    enable_regex: Option<bool>,
    use_text: Option<bool>,
    display_text: Option<String>,
    copy_to_clipboard: Option<bool>,
    clipboard_text: Option<String>,
    use_text_to_voice: Option<bool>,
    interrupt_speech: Option<bool>,
    text_to_voice_text: Option<String>,
    play_media_file: Option<bool>,
    timer_type: Option<String>, // either "Timer" or "NoTimer"? any other possible values?
    timer_name: Option<String>,
    restart_based_on_timer_name: Option<bool>,
    timer_millisecond_duration: Option<i32>,
    timer_duration: Option<i32>,
    timer_visible_duration: Option<i32>,
    timer_start_behavior: Option<String>, // "StartNewTimer" | "ResetTimer" | "IgnoreIfRunning"
    timer_ending_time: Option<i32>,
    use_timer_ending: Option<bool>,
    use_timer_ended: Option<bool>,
    timer_ending_trigger: Option<GINATimerTrigger>,
    timer_ended_trigger: Option<GINATimerTrigger>,
    use_counter_reset_timer: Option<bool>,
    counter_reset_duration: Option<i32>,
    category: Option<String>,
    modified: Option<NaiveDateTime>,
    use_fast_check: Option<bool>,
    timer_early_enders: Vec<GINAEarlyEnder>,
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

#[allow(unused)]
#[derive(Debug, Default)]
struct GINATimerTrigger {
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
#[derive(Debug, Default)]
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

fn parse_trigger_group<R: std::io::Read>(
    parser: &mut xml::reader::EventReader<R>,
) -> GINATriggerGroup {
    let mut trigger_group = GINATriggerGroup::new();
    let mut current_element = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name;
                if current_element == "TriggerGroup" {
                    let nested_group = parse_trigger_group(parser);
                    trigger_group.trigger_groups.push(nested_group);
                } else if current_element == "Trigger" {
                    let trigger = parse_trigger(parser);
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
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    trigger_group
}

fn parse_trigger<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> GINATrigger {
    let mut trigger = GINATrigger::new();
    let mut current_element = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name;
                if current_element == "TimerEndingTrigger" {
                    trigger.timer_ending_trigger = Some(parse_timer_trigger(parser));
                } else if current_element == "TimerEndedTrigger" {
                    trigger.timer_ended_trigger = Some(parse_timer_trigger(parser));
                } else if current_element == "EarlyEnder" {
                    trigger.timer_early_enders.push(parse_early_ender(parser));
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
                "TimerType" => trigger.timer_type = Some(data),
                "TimerName" => trigger.timer_name = Some(data),
                "RestartBasedOnTimerName" => trigger.restart_based_on_timer_name = parse_bool(data),
                "TimerMillisecondDuration" => trigger.timer_millisecond_duration = parse_int(data),
                "TimerDuration" => trigger.timer_duration = parse_int(data),
                "TimerVisibleDuration" => trigger.timer_visible_duration = parse_int(data),
                "TimerStartBehavior" => trigger.timer_start_behavior = Some(data),
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
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    trigger
}

fn parse_timer_trigger<R: std::io::Read>(
    parser: &mut xml::reader::EventReader<R>,
) -> GINATimerTrigger {
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
                if name.local_name == "TimerEndingTrigger" || name.local_name == "TimerEndedTrigger"
                {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    timer_trigger
}

fn parse_early_ender<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> GINAEarlyEnder {
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
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    early_ender
}

fn parse_int(text: String) -> Option<i32> {
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
