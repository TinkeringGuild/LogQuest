use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::template_string::TemplateString};
use async_trait::async_trait;
use std::sync::Arc;

fn copy_to_clipboard(_text: &str) -> EffectResult {
  Ok(())
}

pub struct CopyToClipboardEffect(pub(super) TemplateString);

#[async_trait]
impl ReadyEffect for CopyToClipboardEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let text = self.0.render(&context.match_context);
    copy_to_clipboard(&text)
  }
}
