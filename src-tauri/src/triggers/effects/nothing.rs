use super::{EffectResult, ReadyEffect};
use crate::reactor::EventContext;
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct DoNothingEffect;

#[async_trait]
impl ReadyEffect for DoNothingEffect {
  async fn fire(self: Box<Self>, _context: Arc<EventContext>) -> EffectResult {
    Ok(())
  }
}
