use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::template_string::TemplateString};
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct PlayAudioFileEffect(pub(super) TemplateString);

#[async_trait]
impl ReadyEffect for PlayAudioFileEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let file_path = self.0.render(&context.match_context);
    context
      .mixer
      .play_file(&file_path)
      .await
      .map_err(|e| e.into())
  }
}
