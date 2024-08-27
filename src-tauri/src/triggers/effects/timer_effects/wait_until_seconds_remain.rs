use super::{
  super::{EffectResult, ReadyEffect},
  try_get_timer_context,
};
use crate::{common::timestamp::Timestamp, reactor::EventContext};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::{select, time::Instant};

pub struct WaitUntilSecondsRemainEffect(pub u32);

#[async_trait]
impl ReadyEffect for WaitUntilSecondsRemainEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let timer_context = try_get_timer_context(&context)?;

    let duration_offset = std::time::Duration::from_secs(self.0 as u64);

    let mut end_time_observer = timer_context.end_time.clone();

    let mut wait_until = instant_from_future_timestamp(&end_time_observer.get()) - duration_offset;

    loop {
      select! {
        () = tokio::time::sleep_until(wait_until) => break,

        () = timer_context.finished() => break,

        change = end_time_observer.changed() => match change {
          Ok(()) => {
            wait_until = instant_from_future_timestamp(&end_time_observer.get()) - duration_offset;
          }
          Err(_recv_error) => break
        }
      }
    }

    Ok(())
  }
}

fn instant_from_future_timestamp(future_timestamp: &Timestamp) -> Instant {
  Instant::now() + Timestamp::now().duration_until(future_timestamp)
}
