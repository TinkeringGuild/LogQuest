use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::timers::Timer};
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct StartTimerEffect(pub(super) Timer);

#[async_trait]
impl ReadyEffect for StartTimerEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    context
      .timer_manager
      .start_timer(self.0, context.clone())
      .await;
    Ok(())
  }
}
