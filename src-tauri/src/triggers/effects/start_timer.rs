use super::{EffectResult, ReadyEffect};
use crate::{reactor::EventContext, triggers::timers::Timer};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;

pub(super) struct StartTimerEffect(pub(super) Timer);

#[async_trait]
impl ReadyEffect for StartTimerEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    if let Err(e) = context
      .timer_manager
      .start_timer(self.0, context.clone())
      .await
    {
      error!("StartTimerEffect could not start the timer: {e:?}");
    }
    Ok(())
  }
}
