use super::{EffectResult, ReadyEffect};
use crate::common;
use crate::reactor::ReactorContext;
use async_trait::async_trait;
use std::sync::Arc;

pub(super) struct PauseEffect(pub(super) common::duration::Duration);

#[async_trait]
impl ReadyEffect for PauseEffect {
  async fn fire(self: Box<Self>, _context: Arc<ReactorContext>) -> EffectResult {
    tokio::time::sleep(self.0.into()).await;
    Ok(())
  }
}
