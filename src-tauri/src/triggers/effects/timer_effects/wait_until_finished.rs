use super::{
  super::{EffectResult, ReadyEffect},
  try_get_timer_context,
};
use crate::reactor::EventContext;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WaitUntilFinishedEffect;

#[async_trait]
impl ReadyEffect for WaitUntilFinishedEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    try_get_timer_context(&context)?.finished().await;
    Ok(())
  }
}
