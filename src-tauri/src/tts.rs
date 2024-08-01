use crate::{common::fatal_error, state::state_handle::StateHandle};
use anyhow::bail;
use std::thread;
use tokio::sync::mpsc;
use tracing::error;
use tts::{Gender, Tts};

#[derive(Debug, Clone)]
pub struct Speak {
  text: String,
  interrupt: bool,
  // voice_id: Option<String>,
}
impl Speak {
  pub fn text(text: String) -> Self {
    Speak {
      text,
      interrupt: false,
      // voice_id: None,
    }
  }
}

pub fn spawn(state_handle: StateHandle, rx: mpsc::Receiver<Speak>) -> anyhow::Result<()> {
  let t2s = tts::Tts::default()?;
  thread::Builder::new()
    .name("LogQuest Text-to-Speech".into())
    .spawn(move || thread_loop(t2s, state_handle, rx))?;
  Ok(())
}

fn thread_loop(mut t2s: Tts, _state_handle: StateHandle, mut rx: mpsc::Receiver<Speak>) {
  let voices = t2s.voices();
  if let Err(e) = voices {
    error!("Could not load text-to-speech voices! Error: {e:?}");
  };
  // TODO: I should set voices into the state handle with a mutex lock on it while voices fetch.
  while let Some(Speak {
    text, interrupt, ..
  }) = rx.blocking_recv()
  {
    if let Err(e) = t2s.speak(text.clone(), interrupt) {
      error!(r#"Text-to-Speech engine FAILED to speak: "{text}" [ ERROR: {e:?} ]"#);
    }
  }
}

pub fn speak_once(message: String, voice: Option<String>) -> anyhow::Result<()> {
  let mut t2s = tts::Tts::default()?;
  if let Some(voice_id) = voice {
    let voices = t2s.voices()?;
    if let Some(voice) = voices.iter().find(|v| v.id() == voice_id) {
      t2s.set_voice(voice)?;
    } else {
      bail!("Unknown voice: {voice_id}");
    }
  }
  t2s.speak(message, false)?;
  Ok(())
}

pub fn print_voices() -> anyhow::Result<()> {
  let t2s = tts::Tts::default()?;
  let voices = t2s.voices()?;
  if voices.is_empty() {
    bail!("No voices found!");
  }
  let mut wtr = csv::Writer::from_writer(std::io::stdout());
  wtr.write_record(&["ID", "Language", "Gender"])?;
  for voice in voices.iter() {
    let voice_id = voice.id();
    let voice_language = voice.language().to_string();
    let voice_gender = match voice.gender() {
      Some(Gender::Male) => format!("Male"),
      Some(Gender::Female) => format!("Female"),
      None => "".to_owned(),
    };
    if let Err(_) = wtr.write_record(&[voice_id, voice_language, voice_gender]) {
      fatal_error("Failed to write CSV data to STDOUT");
    }
  }
  Ok(())
}
