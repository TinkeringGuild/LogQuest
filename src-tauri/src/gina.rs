use crate::triggers::{self, Trigger, TriggerGroup};
use crate::triggers::{Matcher, TriggerEffect};
use anyhow::bail;
use chrono::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use xml::reader::XmlEvent;
use zip::read::ZipArchive;

#[allow(unused)]
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

    pub fn to_lq(&self) -> anyhow::Result<Vec<TriggerGroup>> {
        let mut trigger_groups = Vec::with_capacity(self.trigger_groups.len());
        for tg in self.trigger_groups.iter() {
            trigger_groups.push(tg.to_lq()?);
        }
        Ok(trigger_groups)
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

    pub fn to_lq(&self) -> anyhow::Result<triggers::TriggerGroup> {
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

#[allow(unused)]
#[derive(Debug, Default, Serialize, Deserialize)]
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
    modified: Option<NaiveDateTime>,
    timer_early_enders: Vec<GINAEarlyEnder>,

    /// This is ignored during import
    use_fast_check: Option<bool>,

    // TODO: IMPLEMENT THIS WITH TAGS
    category: Option<String>,
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

    /// Converts this GINATrigger to a LogQuest Trigger
    pub fn to_lq(&self) -> anyhow::Result<triggers::Trigger> {
        let trigger_name = self.name.clone().unwrap_or_else(|| untitled("Trigger"));
        Ok(Trigger {
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
                    vec![triggers::Matcher::GINAPattern(text.to_owned())]
                }
                (Some(text), Some(false)) | (Some(text), None) => {
                    vec![Matcher::WholeLine(text.to_owned())]
                }
                _ => bail!("Cannot interpret GINA trigger text for {}", &trigger_name),
            },
            effects: {
                // TODO: render the name of the timer here with TemplateString
                let timer_name = self.timer_name.clone().unwrap_or_else(|| untitled("Timer"));

                let display_text: Option<TriggerEffect> = effect_from_options(
                    &self.use_text,
                    &self.display_text,
                    TriggerEffect::OverlayMessage,
                );

                let copy_text: Option<TriggerEffect> = effect_from_options(
                    &self.copy_to_clipboard,
                    &self.clipboard_text,
                    TriggerEffect::CopyToClipboard,
                );

                // TODO: This needs to handle self.interrupt_speech
                let tts: Option<TriggerEffect> = effect_from_options(
                    &self.use_text_to_voice,
                    &self.text_to_voice_text,
                    TriggerEffect::TextToSpeech,
                );

                let play_sound_file: Option<TriggerEffect> = match self.play_media_file {
                    Some(true) => Some(TriggerEffect::PlayAudioFile(None)), // the XML does not include the sound file's filepath
                    _ => None,
                };

                let timer: Option<TriggerEffect> = match self.timer_type {
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
                                _ => bail!(
                                    "Could not determine Timer duration for timer {timer_name}!",
                                ),
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
                                            triggers::TimerEffect::AddTag(
                                                triggers::TimerTag::ending(),
                                            ),
                                        ];

                                        if let (Some(true), Some(ending)) =
                                            (self.use_timer_ending, &self.timer_ending_trigger)
                                        {
                                            if let Some(singularized) = singularize_effects(
                                                ending.to_lq(),
                                                triggers::TimerEffect::Parallel,
                                            ) {
                                                seq.push(singularized);
                                            }
                                        }
                                        updates.push(triggers::TimerEffect::Sequence(seq));
                                    }
                                }

                                // Timer Ended with WaitUntilFinished and Parallel effects
                                if let (Some(true), Some(ended)) =
                                    (self.use_timer_ended, &self.timer_ended_trigger)
                                {
                                    if let Some(singularized) = singularize_effects(
                                        ended.to_lq(),
                                        triggers::TimerEffect::Parallel,
                                    ) {
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
                                triggers::TimerStartPolicy::StartAndReplacesAnyTimerWithName(
                                    timer_name,
                                )
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
                        Some(TriggerEffect::StartTimer { timer, policy })
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
        let mut enders_filter: triggers::Filter = Vec::with_capacity(self.timer_early_enders.len());
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
impl GINATimerStartBehavior {
    pub fn to_lq(&self) -> triggers::TimerStartBehavior {
        match self {
            Self::StartNewTimer => triggers::TimerStartBehavior::StartNewTimer,
            Self::RestartTimer => triggers::TimerStartBehavior::RestartTimer,
            Self::IgnoreIfRunning => triggers::TimerStartBehavior::IgnoreIfRunning,
        }
    }
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

    fn to_lq(&self) -> anyhow::Result<triggers::Matcher> {
        Ok(match (self.enable_regex, self.early_end_text.clone()) {
            (Some(true), Some(pattern)) => triggers::Matcher::GINAPattern(pattern),
            (Some(false), Some(line)) => triggers::Matcher::WholeLine(line),
            _ => bail!("Invalid Early Ender"),
        })
    }
}

pub fn load_gina_triggers_from_file_path(file_path: &PathBuf) -> anyhow::Result<GINATriggers> {
    let shared_data = match file_path.extension().and_then(|s| s.to_str()) {
        Some("gtp") => {
            let file = File::open(file_path)?;
            let mut archive = ZipArchive::new(file)?;
            let share_data_xml = archive.by_name("ShareData.xml").map_err(|_| {
                anyhow::anyhow!("Could not find a ShareData.xml file in the GTP archive")
            })?;
            let mut reader = BufReader::new(share_data_xml);
            read_xml(&mut reader)?
        }
        Some("xml") => {
            let file = File::open(file_path)?;
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
                    let trigger_group = parse_trigger_group(&mut parser)?;
                    shared_data.trigger_groups.push(trigger_group);
                }
            }
            Ok(XmlEvent::EndDocument) => break,
            Err(e) => bail!(e),
            _ => {}
        }
    }

    Ok(shared_data)
}

fn effect_from_options<F>(
    condition: &Option<bool>,
    text: &Option<String>,
    converter: F,
) -> Option<TriggerEffect>
where
    F: FnOnce(triggers::TemplateString) -> TriggerEffect,
{
    match (condition, text.as_deref()) {
        (Some(true), Some("")) => None,
        (Some(true), Some(text)) => Some(converter(text.into())),
        _ => None,
    }
}

fn untitled(what: &str) -> String {
    format!("Untitled {} [{}]", what, random_id(4))
}

fn random_id(length: u8) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn parse_trigger_group<R: std::io::Read>(
    parser: &mut xml::reader::EventReader<R>,
) -> anyhow::Result<GINATriggerGroup> {
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
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(trigger_group)
}

fn parse_trigger<R: std::io::Read>(
    parser: &mut xml::reader::EventReader<R>,
) -> anyhow::Result<GINATrigger> {
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
                "TimerType" => {
                    trigger.timer_type = Some(match data.as_str() {
                        "Timer" => GINATimerType::Timer,
                        "NoTimer" => GINATimerType::NoTimer,
                        "Stopwatch" => GINATimerType::Stopwatch,
                        "RepeatingTimer" => GINATimerType::RepeatingTimer,
                        _ => bail!("Unrecognized TimerType: {}", data),
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
                        _ => bail!("Unrecognized GINA start behavior: {}", data),
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
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(trigger)
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
