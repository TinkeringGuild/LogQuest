use super::{EffectResult, ReadyEffect};
use crate::reactor::ReactorContext;
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct DoNothingEffect;

#[async_trait]
impl ReadyEffect for DoNothingEffect {
  async fn fire(self: Box<Self>, _context: Arc<ReactorContext>) -> EffectResult {
    Ok(())
  }
}
