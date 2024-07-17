use std::str::FromStr;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize, Serializer};

pub type Filter = Vec<Matcher>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Matcher {
    WholeLine(String),
    PartialLine(String),
    Pattern(SerializableRegex), // TODO: this should have params extracted from it
    GINAPattern(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerStartPolicy {
    AlwaysStartNewTimer,
    DoNothingIfTimerRunning,
    StartAndReplacesAllTimers,
    StartAndReplacesAnyTimerWithName(String), // maybe this should be TemplateString?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerEffect {
    Parallel(Vec<TriggerEffect>),
    Sequence(Vec<TriggerEffect>),
    /// This uses an Option<String> because importing from GINA does not include
    /// a reference to the sound file, but the TriggerEffect should be preserved when
    /// importing to allow the user to select a file during/after import.
    PlayAudioFile(Option<TemplateString>),
    CopyToClipboard(TemplateString),
    OverlayMessage(TemplateString),
    TextToSpeech(TemplateString),
    StartTimer {
        timer: Timer,
        policy: TimerStartPolicy,
    },
    StartStopwatch(Stopwatch),
    RunSystemCommand(CommandTemplate),

    /// This is meant to be used within a Sequence
    Pause(Duration),
    /// Useful for temporarily disabling an effect or use as a default Effect
    DoNothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerEffect {
    Parallel(Vec<TimerEffect>),
    Sequence(Vec<TimerEffect>),
    Pause(Duration),
    PlayAudioFile(Option<TemplateString>),
    DoNothing,
    OverlayMessage(TemplateString),
    Speak(TemplateString),
    SpeakStop,

    HideTimer,
    RestartTimer,
    IncrementCounter,
    DecrementCounter,
    ResetCounter,
    AddTag(TimerTag),
    RemoveTag(TimerTag),
    WaitUntilTagged(TimerTag),
    WaitUntilSecondsRemain(u32),
    WaitUntilFilterMatches(Filter),
    // WaitUntilRestarted,
    WaitUntilFinished,
    ClearTimer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerGroup {
    pub name: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub children: Vec<TriggerGroupDescendant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerGroupDescendant {
    T(Trigger),
    TG(TriggerGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub comment: Option<String>,
    pub enabled: bool,
    pub filter: Filter,
    pub effects: Vec<TriggerEffect>,
    pub last_modified: DateTime<Utc>, // tags: Vec<Tag>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stopwatch {
    pub name: String,
    pub tags: Vec<TimerTag>,
    pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub name: String,
    pub tags: Vec<TimerTag>,
    pub duration: Duration,
    pub timer_start_behavior: TimerStartBehavior,

    /// When finished, the timer starts over until terminated
    pub repeats: bool,

    pub updates: Vec<TimerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerStartBehavior {
    StartNewTimer,
    RestartTimer,
    IgnoreIfRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerTag(String);
impl TimerTag {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// used for marking a Timer has entered the "ending" state
    pub fn ending() -> Self {
        Self::new("ENDING")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateString {
    tmpl: String,
    param_names: Vec<String>,
}

impl From<&str> for TemplateString {
    fn from(tmpl: &str) -> Self {
        // TODO: Compile this regex at compilation time with lazy_static or a macro
        let vars_regex: Regex = Regex::new(r"\$\{\s*([\w_-]+)\s*\}").unwrap();
        let param_names: Vec<String> = vars_regex
            .captures_iter(tmpl)
            .filter_map(|capture| capture.get(1).map(|c| c.as_str().to_owned()))
            .collect();
        TemplateString {
            tmpl: tmpl.to_owned(),
            param_names,
        }
    }
}
impl From<String> for TemplateString {
    fn from(tmpl: String) -> Self {
        tmpl.as_str().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTemplate {
    pub command: TemplateString,
    pub params: Vec<TemplateString>,
    pub write_to_stdin: Option<TemplateString>,
}

#[derive(Debug, Clone)]
pub struct Duration(u32);
impl Duration {
    pub fn from_millis(millis: u32) -> Self {
        Duration(millis)
    }
    pub fn from_secs(secs: u32) -> Self {
        Duration(secs * 1000)
    }
}
impl Into<std::time::Duration> for Duration {
    fn into(self) -> std::time::Duration {
        std::time::Duration::from_millis(self.0.into())
    }
}
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: u32 = Deserialize::deserialize(deserializer)?;
        Ok(Duration(value))
    }
}

// TODO: This needs to have named parameters extracted from it.
#[derive(Debug, Clone)]
pub struct SerializableRegex(Regex);

impl Serialize for SerializableRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for SerializableRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RegexVisitor;
        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = SerializableRegex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid regex pattern")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Regex::from_str(value)
                    .map(SerializableRegex)
                    .map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(RegexVisitor)
    }
}
