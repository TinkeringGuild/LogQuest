use super::{
  super::{EffectResult, ReadyEffect},
  try_get_timer_context,
};
use crate::reactor::EventContext;
use async_trait::async_trait;
use std::sync::Arc;

pub struct HideTimerEffect;
pub struct UnhideTimerEffect;

#[async_trait]
impl ReadyEffect for HideTimerEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    try_get_timer_context(&context)?.set_is_hidden(true).await?;
    Ok(())
  }
}

#[async_trait]
impl ReadyEffect for UnhideTimerEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    try_get_timer_context(&context)?
      .set_is_hidden(false)
      .await?;
    Ok(())
  }
}
