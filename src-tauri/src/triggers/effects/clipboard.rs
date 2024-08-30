use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::template_string::TemplateString};
use async_trait::async_trait;
use std::sync::Arc;

pub struct CopyToClipboardEffect(pub(super) TemplateString);

#[async_trait]
impl ReadyEffect for CopyToClipboardEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let text = self.0.render(&context.match_context);
    context.clipboard.write_text(&text).await;
    Ok(())
  }
}
