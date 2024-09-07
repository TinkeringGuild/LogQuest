use crate::{common::fatal_error, state::state_handle::StateHandle};
use std::{
  collections::LinkedList,
  sync::{Arc, Mutex},
  thread,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};
use tts::{Gender, Tts};

#[derive(Debug, Clone)]
pub enum TTS {
  Speak {
    text: String,
    interrupt: bool,
    tx_done: Arc<oneshot::Sender<()>>,
  },
  StopSpeaking,
  SetVoice(String),
  Quit,
}

#[derive(thiserror::Error, Debug)]
pub enum SpeakError {
  #[error("Unknown voice: {0}")]
  UnknownVoice(String),
  #[error(transparent)]
  TTS(#[from] tts::Error),
}

pub fn spawn(state_handle: StateHandle, rx: mpsc::Receiver<TTS>) -> Result<(), tts::Error> {
  let t2s = tts::Tts::default()?;
  thread::Builder::new()
    .name("LogQuest Text-to-Speech".into())
    .spawn(move || thread_loop(t2s, state_handle, rx))
    .expect("Could not spawn a thread for the TTS engine"); // panic-worthy
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

  let locked_done_sender_queue =
    Arc::new(Mutex::new(LinkedList::<Arc<oneshot::Sender<()>>>::new()));

  let locked_done_sender_queue_ = locked_done_sender_queue.clone();
  let utterance_end_callback = move |_| {
    let mut queue = locked_done_sender_queue_
      .lock()
      .expect("TTS futures queue poisoned!");
    let Some(next_sender) = queue.pop_front() else {
      error!("Found no callback to send the TTS completion notification to!");
      return;
    };
    let next_sender = Arc::into_inner(next_sender).unwrap();
    _ = next_sender.send(());
  };

  if let Err(e) = t2s.on_utterance_end(Some(Box::new(utterance_end_callback))) {
    error!("Could not bind on_utterance_end callback on the TTS engine: {e:?}");
    return;
  }

  loop {
    match rx.blocking_recv() {
      Some(TTS::Quit) => {
        debug!("TTS loop QUITTING");
        break;
      }
      Some(TTS::Speak {
        text,
        interrupt,
        tx_done,
      }) => {
        let mut queue = locked_done_sender_queue
          .lock()
          .expect("TTS futures queue poisoned!");
        queue.push_back(tx_done);
        if let Err(e) = t2s.speak(text.clone(), interrupt) {
          error!(r#"Text-to-Speech engine FAILED to speak: "{text}" [ ERROR: {e:?} ]"#);
          let tx_done = Arc::into_inner(queue.pop_back().unwrap()).unwrap();
          _ = tx_done.send(());
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
      Some(TTS::StopSpeaking) => {
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

/// This function is designed to be used from the CLI, not via the reactor.
pub fn speak_once(message: String, voice: Option<String>) -> Result<(), SpeakError> {
  let mut t2s = tts::Tts::default()?;
  if let Some(voice_id) = voice {
    let voices = t2s.voices()?;
    if let Some(voice) = voices.iter().find(|v| v.id() == voice_id) {
      t2s.set_voice(voice)?;
    } else {
      return Err(SpeakError::UnknownVoice(voice_id));
    }
  }
  t2s.speak(message, false)?;
  Ok(())
}

/// This function is designed to be used from CLI, so it will exit the process if
/// an error is encountered.
pub fn print_voices() {
  let Ok(t2s) = tts::Tts::default() else {
    fatal_error("Could not create TTS engine!");
  };
  let Ok(voices) = t2s.voices() else {
    fatal_error("Could not retrieve TTS voices!");
  };
  if voices.is_empty() {
    println!("No voices found in the TTS engine!");
    return;
  }
  let mut wtr = csv::Writer::from_writer(std::io::stdout());
  if let Err(e) = wtr.write_record(&["ID", "Language", "Gender"]) {
    fatal_error(format!("Could not write CSV output: {e:?}"));
  }
  for voice in voices.iter() {
    let voice_id = voice.id();
    let voice_language = voice.language().to_string();
    let voice_gender = String::from(match voice.gender() {
      Some(Gender::Male) => "Male",
      Some(Gender::Female) => "Female",
      None => "",
    });
    if let Err(e) = wtr.write_record(&[voice_id, voice_language, voice_gender]) {
      fatal_error(format!("Failed to write CSV data to STDOUT: {e:?}"));
    }
  }
}
