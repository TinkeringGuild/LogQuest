use super::{EffectError, EffectResult, EffectTemplate, ReadyEffect};
use crate::matchers::MatchContext;
use async_trait::async_trait;
use futures::future::join_all;
use tauri::async_runtime::{spawn, JoinHandle};

struct TemplateParallel(Vec<Box<dyn EffectTemplate>>);
struct EffectParallel(Vec<Box<dyn ReadyEffect>>);

impl EffectTemplate for TemplateParallel {
  fn ready(&self, context: &MatchContext) -> Box<dyn ReadyEffect> {
    let ready_effects = self
      .0
      .iter()
      .map(|template| template.ready(context))
      .collect::<Vec<_>>();
    Box::new(EffectParallel(ready_effects))
  }
}

#[async_trait]
impl ReadyEffect for EffectParallel {
  async fn fire(self: Box<Self>) -> EffectResult {
    let join_handles: Vec<JoinHandle<EffectResult>> = self
      .0
      .into_iter()
      .map(|effect| spawn(async move { effect.fire().await }))
      .collect();

    let errors = join_all(join_handles)
      .await
      .into_iter()
      .filter_map(|tauri_error| match tauri_error {
        Ok(Ok(())) => None,
        Ok(Err(effect_error)) => Some(effect_error),
        Err(tauri_error) => Some(tauri_error.into()),
      })
      .collect::<Vec<_>>();

    match errors.len() {
      0 => Ok(()),
      1 => Err(errors.into_iter().next().unwrap()),
      _ => Err(EffectError::Multiple(errors)),
    }
  }
}
