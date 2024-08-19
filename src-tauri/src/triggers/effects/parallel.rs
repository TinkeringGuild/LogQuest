use super::{EffectError, EffectResult, ReadyEffect};
use crate::reactor::ReactorContext;
use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;
use tauri::async_runtime::{spawn, JoinHandle};

pub(super) struct EffectParallel(pub(super) Vec<Box<dyn ReadyEffect>>);

#[async_trait]
impl ReadyEffect for EffectParallel {
  async fn fire(self: Box<Self>, context: Arc<ReactorContext>) -> EffectResult {
    let contexts = (0..self.0.len()).map(|_| context.clone());

    let join_handles: Vec<JoinHandle<EffectResult>> = self
      .0
      .into_iter()
      .zip(contexts)
      .map(|(effect, ctx)| spawn(async move { effect.fire(ctx).await }))
      .collect();

    let errors = join_all(join_handles)
      .await
      .into_iter()
      .filter_map(|tauri_error| match tauri_error {
        Ok(Ok(())) => None,
        Ok(Err(effect_error)) => Some(effect_error),
        Err(tauri_error) => Some(tauri_error.into()),
      })
      .collect::<Vec<EffectError>>();

    match errors.len() {
      0 => Ok(()),
      1 => Err(errors.into_iter().next().unwrap()),
      _ => Err(EffectError::Multiple(errors)),
    }
  }
}
