use super::{EffectResult, ReadyEffect};
use crate::reactor::EventContext;
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct EffectSequence(pub(super) Vec<Box<dyn ReadyEffect>>);

#[async_trait]
impl ReadyEffect for EffectSequence {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    for effect in self.0.into_iter() {
      effect.fire(context.clone()).await?;
    }
    Ok(())
  }
}
