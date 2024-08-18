use super::{EffectResult, EffectTemplate, ReadyEffect};
use crate::matchers::MatchContext;
use crate::triggers::template_string::TemplateString;
use async_trait::async_trait;

fn copy_to_clipboard(_text: &str) -> EffectResult {
  Ok(())
}

pub struct CopyToClipboardTemplate(TemplateString);
pub struct CopyToClipboardEffect(String);

impl EffectTemplate for CopyToClipboardTemplate {
  fn ready(&self, context: &MatchContext) -> Box<dyn ReadyEffect> {
    let text = self.0.render(&context);
    Box::new(CopyToClipboardEffect(text))
  }
}

#[async_trait]
impl ReadyEffect for CopyToClipboardEffect {
  async fn fire(self: Box<Self>) -> EffectResult {
    copy_to_clipboard(&self.0)
  }
}
