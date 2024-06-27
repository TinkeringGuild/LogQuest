use anyhow::bail;
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
    trigger_groups: Vec<TriggerGroup>,
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
struct TriggerGroup {
    name: Option<String>,
    comments: Option<String>,
    self_commented: Option<String>,
    group_id: Option<String>,
    enable_by_default: Option<String>,
    trigger_groups: Vec<TriggerGroup>,
    triggers: Vec<Trigger>,
}

impl TriggerGroup {
    fn new() -> Self {
        TriggerGroup {
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
struct Trigger {
    name: Option<String>,
    trigger_text: Option<String>,
    comments: Option<String>,
    enable_regex: Option<String>,
    use_text: Option<String>,
    display_text: Option<String>,
    copy_to_clipboard: Option<String>,
    clipboard_text: Option<String>,
    use_text_to_voice: Option<String>,
    interrupt_speech: Option<String>,
    text_to_voice_text: Option<String>,
    play_media_file: Option<String>,
    timer_type: Option<String>,
    timer_name: Option<String>,
    restart_based_on_timer_name: Option<String>,
    timer_millisecond_duration: Option<String>,
    timer_duration: Option<String>,
    timer_visible_duration: Option<String>,
    timer_start_behavior: Option<String>,
    timer_ending_time: Option<String>,
    use_timer_ending: Option<String>,
    use_timer_ended: Option<String>,
    timer_ending_trigger: Option<TimerTrigger>,
    timer_ended_trigger: Option<TimerTrigger>,
    use_counter_reset_timer: Option<String>,
    counter_reset_duration: Option<String>,
    category: Option<String>,
    modified: Option<String>,
    use_fast_check: Option<String>,
    timer_early_enders: Vec<EarlyEnder>,
}

impl Trigger {
    fn new() -> Self {
        Trigger {
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
struct TimerTrigger {
    use_text: Option<String>,
    display_text: Option<String>,
    use_text_to_voice: Option<String>,
    interrupt_speech: Option<String>,
    text_to_voice_text: Option<String>,
    play_media_file: Option<String>,
}

impl TimerTrigger {
    fn new() -> Self {
        TimerTrigger {
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
struct EarlyEnder {
    early_end_text: Option<String>,
    enable_regex: Option<String>,
}

impl EarlyEnder {
    fn new() -> Self {
        EarlyEnder {
            early_end_text: None,
            enable_regex: None,
        }
    }
}

fn parse_trigger_group<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> TriggerGroup {
    let mut trigger_group = TriggerGroup::new();
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
                "SelfCommented" => trigger_group.self_commented = Some(data),
                "GroupId" => trigger_group.group_id = Some(data),
                "EnableByDefault" => trigger_group.enable_by_default = Some(data),
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

fn parse_trigger<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> Trigger {
    let mut trigger = Trigger::new();
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
                "EnableRegex" => trigger.enable_regex = Some(data),
                "UseText" => trigger.use_text = Some(data),
                "DisplayText" => trigger.display_text = Some(data),
                "CopyToClipboard" => trigger.copy_to_clipboard = Some(data),
                "ClipboardText" => trigger.clipboard_text = Some(data),
                "UseTextToVoice" => trigger.use_text_to_voice = Some(data),
                "InterruptSpeech" => trigger.interrupt_speech = Some(data),
                "TextToVoiceText" => trigger.text_to_voice_text = Some(data),
                "PlayMediaFile" => trigger.play_media_file = Some(data),
                "TimerType" => trigger.timer_type = Some(data),
                "TimerName" => trigger.timer_name = Some(data),
                "RestartBasedOnTimerName" => trigger.restart_based_on_timer_name = Some(data),
                "TimerMillisecondDuration" => trigger.timer_millisecond_duration = Some(data),
                "TimerDuration" => trigger.timer_duration = Some(data),
                "TimerVisibleDuration" => trigger.timer_visible_duration = Some(data),
                "TimerStartBehavior" => trigger.timer_start_behavior = Some(data),
                "TimerEndingTime" => trigger.timer_ending_time = Some(data),
                "UseTimerEnding" => trigger.use_timer_ending = Some(data),
                "UseTimerEnded" => trigger.use_timer_ended = Some(data),
                "UseCounterResetTimer" => trigger.use_counter_reset_timer = Some(data),
                "CounterResetDuration" => trigger.counter_reset_duration = Some(data),
                "Category" => trigger.category = Some(data),
                "Modified" => trigger.modified = Some(data),
                "UseFastCheck" => trigger.use_fast_check = Some(data),
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

fn parse_timer_trigger<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> TimerTrigger {
    let mut timer_trigger = TimerTrigger::new();
    let mut current_element = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name;
            }
            Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
                "UseText" => timer_trigger.use_text = Some(data),
                "DisplayText" => timer_trigger.display_text = Some(data),
                "UseTextToVoice" => timer_trigger.use_text_to_voice = Some(data),
                "InterruptSpeech" => timer_trigger.interrupt_speech = Some(data),
                "TextToVoiceText" => timer_trigger.text_to_voice_text = Some(data),
                "PlayMediaFile" => timer_trigger.play_media_file = Some(data),
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

fn parse_early_ender<R: std::io::Read>(parser: &mut xml::reader::EventReader<R>) -> EarlyEnder {
    let mut early_ender = EarlyEnder::new();
    let mut current_element = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name;
            }
            Ok(XmlEvent::Characters(data)) => match current_element.as_str() {
                "EarlyEndText" => early_ender.early_end_text = Some(data),
                "EnableRegex" => early_ender.enable_regex = Some(data),
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
