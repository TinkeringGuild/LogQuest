use super::{
  super::{EffectResult, ReadyEffect},
  try_get_timer_context,
};
use crate::{
  logs::log_line_stream::LogLineStream, matchers::FilterWithContext, reactor::EventContext,
};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::{
  select,
  time::{sleep_until, Instant},
};

pub struct WaitUntilFilterMatchesTimerEffect(
  pub FilterWithContext,
  pub Option<crate::common::duration::Duration>,
);

#[async_trait]
impl ReadyEffect for WaitUntilFilterMatchesTimerEffect {
  async fn fire(self: Box<Self>, context: Arc<EventContext>) -> EffectResult {
    let timer_context = try_get_timer_context(&context)?;

    let subscription = context.tx_log_file_events.subscribe();
    let mut stream = LogLineStream::create(&context.cursor_after, subscription).await?;

    let filter = self.0.compile_with_context(&context.match_context);

    let timeout_instant_maybe: Option<Instant> = self.1.map(|d| Instant::now() + d.into());

    loop {
      select! {
        () = async {
          sleep_until(timeout_instant_maybe.unwrap()).await // unwrap is infallible here due to is_some() check
        }, if timeout_instant_maybe.is_some() => break,

        () = timer_context.finished() => break,

        next = stream.next() => match next {
          None => break,
          Some((line, _cursor_after)) => {
            let character_name = &context.match_context.character_name;
            if filter.check(&line.content, character_name).is_some() {
              break;
            }
          }
        }
      }
    }

    Ok(())
  }
}
