pub mod clear;
pub mod hide;
pub mod restart;
pub mod wait_until_filter_matches;
pub mod wait_until_finished;
pub mod wait_until_seconds_remain;

use super::EffectError;
use crate::{reactor::EventContext, state::timer_manager::TimerContext};
use std::sync::Arc;

fn try_get_timer_context(event_context: &Arc<EventContext>) -> Result<&TimerContext, EffectError> {
  match &event_context.timer_context {
    Some(timer_context) => Ok(timer_context),
    None => Err(EffectError::TimerEffectWithoutTimerContext),
  }
}
