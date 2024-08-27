use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::template_string::TemplateString, tts::TTS};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{error, warn};

pub(super) struct SpeakEffect {
  pub(super) tmpl: TemplateString,
  pub(super) interrupt: bool,
  pub(super) non_blocking: bool,
}

pub(super) struct SpeakStopEffect;

#[async_trait]
impl ReadyEffect for SpeakEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let message = self.tmpl.render(&context.match_context);

    let (tx_done, rx_done) = oneshot::channel::<()>();

    if let Err(_) = context
      .t2s_tx
      .send(TTS::Speak {
        text: message.clone(),
        interrupt: self.interrupt,
        tx_done: Arc::new(tx_done),
      })
      .await
    {
      error!(r#"Text-to-Speech channel closed! Ignoring TTS message: "{message}""#);
    }

    if !self.non_blocking {
      let _ = rx_done.await;
    }

    Ok(())
  }
}

#[async_trait]
impl ReadyEffect for SpeakStopEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    if let Err(_send_error) = context.t2s_tx.send(TTS::StopSpeaking).await {
      warn!("Cannot send StopSpeaking effect to the Text-to-Speech engine");
    }
    Ok(())
  }
}
