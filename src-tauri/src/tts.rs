use crate::state::state_handle::StateHandle;
use std::thread;
use tokio::sync::mpsc;
use tracing::error;
use tts::Tts;

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
  while let Some(Speak {
    text, interrupt, ..
  }) = rx.blocking_recv()
  {
    if let Err(e) = t2s.speak(text.clone(), interrupt) {
      error!(r#"Text-to-Speech engine FAILED to speak: "{text}" [ ERROR: {e:?} ]"#);
    }
  }
}
