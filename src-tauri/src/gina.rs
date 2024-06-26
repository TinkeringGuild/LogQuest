use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use xml::reader::{EventReader, XmlEvent};

pub fn parse_gina_trigger_xml(xml_file: PathBuf) {
    let file = File::open(xml_file).unwrap();
    let buf_reader = BufReader::new(file);
    let mut parser = EventReader::new(buf_reader);

    let mut shared_data = SharedData::new();

    #[allow(unused_assignments)] // initial assigned value of current_element isn't used
    let mut current_element: String = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name.clone();
                if current_element == "TriggerGroup" {
                    if shared_data.trigger_groups.is_none() {
                        shared_data.trigger_groups = Some(Vec::new());
                    }
                    let trigger_group = parse_trigger_group(&mut parser);
                    shared_data
                        .trigger_groups
                        .as_mut()
                        .unwrap()
                        .push(trigger_group);
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

    println!("{:#?}", shared_data);
}

#[allow(unused)]
#[derive(Debug, Default)]
struct SharedData {
    trigger_groups: Option<Vec<TriggerGroup>>,
}

impl SharedData {
    fn new() -> Self {
        SharedData {
            trigger_groups: None,
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
    trigger_groups: Option<Vec<TriggerGroup>>,
    triggers: Option<Vec<Trigger>>,
}

impl TriggerGroup {
    fn new() -> Self {
        TriggerGroup {
            name: None,
            comments: None,
            self_commented: None,
            group_id: None,
            enable_by_default: None,
            trigger_groups: None,
            triggers: None,
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
    timer_early_enders: Option<Vec<EarlyEnder>>,
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
            timer_early_enders: None,
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

fn parse_trigger_group<R: std::io::Read>(parser: &mut EventReader<R>) -> TriggerGroup {
    let mut trigger_group = TriggerGroup::new();
    let mut current_element = String::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = name.local_name.clone();
                if current_element == "TriggerGroup" {
                    if trigger_group.trigger_groups.is_none() {
                        trigger_group.trigger_groups = Some(Vec::new());
                    }
                    let nested_group = parse_trigger_group(parser);
                    trigger_group
                        .trigger_groups
                        .as_mut()
                        .unwrap()
                        .push(nested_group);
                } else if current_element == "Trigger" {
                    if trigger_group.triggers.is_none() {
                        trigger_group.triggers = Some(Vec::new());
                    }
                    let trigger = parse_trigger(parser);
                    trigger_group.triggers.as_mut().unwrap().push(trigger);
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

fn parse_trigger<R: std::io::Read>(parser: &mut EventReader<R>) -> Trigger {
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
                    if trigger.timer_early_enders.is_none() {
                        trigger.timer_early_enders = Some(Vec::new());
                    }
                    trigger
                        .timer_early_enders
                        .as_mut()
                        .unwrap()
                        .push(parse_early_ender(parser));
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

fn parse_timer_trigger<R: std::io::Read>(parser: &mut EventReader<R>) -> TimerTrigger {
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

fn parse_early_ender<R: std::io::Read>(parser: &mut EventReader<R>) -> EarlyEnder {
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
