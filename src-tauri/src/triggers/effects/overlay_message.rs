use super::{EffectResult, ReadyEffect};
use crate::{reactor::ReactorContext, triggers::template_string::TemplateString};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

pub(super) struct OverlayMessageEffect(pub(super) TemplateString);

#[async_trait]
impl ReadyEffect for OverlayMessageEffect {
  async fn fire(self: Box<Self>, context: Arc<ReactorContext>) -> EffectResult {
    let message = self.0.render(&context.match_context);
    info!(r#"TriggerEffect::OverlayMessage("{message}")"#);
    context.overlay_manager.message(message);
    Ok(())
  }
}
