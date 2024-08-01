use crate::{common::fatal_error, state::state_handle::StateHandle};
use anyhow::bail;
use std::thread;
use tokio::sync::mpsc;
use tracing::{error, info};
use tts::{Gender, Tts};

#[derive(Debug, Clone)]
pub enum TTS {
  Interrupt,
  Speak { text: String, interrupt: bool },
  SetVoice(String),
}

pub fn spawn(state_handle: StateHandle, rx: mpsc::Receiver<TTS>) -> anyhow::Result<()> {
  let t2s = tts::Tts::default()?;
  thread::Builder::new()
    .name("LogQuest Text-to-Speech".into())
    .spawn(move || thread_loop(t2s, state_handle, rx))?;
  Ok(())
}

fn thread_loop(mut t2s: Tts, _state_handle: StateHandle, mut rx: mpsc::Receiver<TTS>) {
  // TODO: I should set voices into the state handle with a mutex lock on it while voices fetch.
  let voices = match t2s.voices() {
    Ok(v) => v,
    Err(e) => {
      error!("Could not load text-to-speech voices! [ ERROR: {e:?} ]");
      return;
    }
  };
  loop {
    match rx.blocking_recv() {
      Some(TTS::Speak { text, interrupt }) => {
        if let Err(e) = t2s.speak(text.clone(), interrupt) {
          error!(r#"Text-to-Speech engine FAILED to speak: "{text}" [ ERROR: {e:?} ]"#);
        }
      }
      Some(TTS::SetVoice(voice_id)) => {
        // TODO: IF set_voice() IS CALLED ON THE TTS ENGINE WHILE IT IS SPEAKING, DOES THAT
        // AFFECT THE CURRENT SPEECH OR DOES IT ONLY AFFECT NEW CALLS TO SPEAK?
        let Some(voice) = voices.iter().find(|v| v.id() == voice_id) else {
          error!("Tried setting voice to an unknown ID: {voice_id}");
          continue;
        };
        if let Err(e) = t2s.set_voice(voice) {
          error!(r#"Could not set voice to voice ID "{voice_id}" [ ERROR: {e:?} ]"#);
        }
      }
      Some(TTS::Interrupt) => {
        if let Err(e) = t2s.stop() {
          error!("Could not stop the text-to-speech engine! [ ERROR: {e:?} ]");
        }
      }
      None => {
        info!("Text-to-Speech engine stopping (channel closed)");
        return;
      }
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
