use super::{EffectResult, EffectTemplate, ReadyEffect};
use crate::matchers::MatchContext;
use async_trait::async_trait;

struct EffectSequence(Vec<Box<dyn ReadyEffect>>);

#[async_trait]
impl ReadyEffect for EffectSequence {
  async fn fire(self: Box<Self>) -> EffectResult {
    for effect in self.0.into_iter() {
      effect.fire().await?;
    }
    Ok(())
  }
}

struct TemplateSequence(Vec<Box<dyn EffectTemplate>>);

impl EffectTemplate for TemplateSequence {
  fn ready(&self, context: &MatchContext) -> Box<dyn ReadyEffect> {
    let ready_effects = self
      .0
      .iter()
      .map(|template| template.ready(context))
      .collect::<Vec<_>>();
    Box::new(EffectSequence(ready_effects))
  }
}
