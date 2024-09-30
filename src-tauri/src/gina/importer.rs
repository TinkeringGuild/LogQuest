use super::{
  conversion::GINAConversionError,
  xml::{load_gina_triggers_from_file_path, GINAParseError},
};
use crate::{
  common::{
    progress_reporter::{ProgressReporter, ProgressUpdate},
    timestamp::Timestamp,
  },
  state::state_handle::StateHandle,
};
use std::{path::Path, sync::Arc};
use tokio::sync::{oneshot, watch};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum GINAImportError {
  #[error("GINA conversion error")]
  ConversionError(#[from] GINAConversionError),
  #[error("GINA parse error")]
  ParseError(#[from] GINAParseError),
}

pub fn import_from_gina_export_file(
  file_path: &Path,
  state: StateHandle,
) -> (
  Arc<ProgressReporter>,
  watch::Receiver<ProgressUpdate>,
  oneshot::Receiver<Result<(), GINAImportError>>,
) {
  let (progress_reporter, rx_progess_update) = ProgressReporter::new();
  let progress_reporter = Arc::new(progress_reporter);

  let (tx_result, rx_result) = oneshot::channel::<Result<(), GINAImportError>>();

  let file_path = file_path.to_owned();
  let progress_reporter_ = progress_reporter.clone();

  std::thread::Builder::new()
    .name("LogQuest GINA Import".into())
    .spawn(move || {
      state.bulk_update_triggers(|index| {
        let import_time: Timestamp = Timestamp::now();
        progress_reporter_.update("Parsing GINA XML");
        let from_gina = match load_gina_triggers_from_file_path(&file_path, &progress_reporter_) {
          Ok(value) => value,
          Err(e) => {
            error!("Encountered error parsing GINA import file: {e:?}");
            _ = tx_result.send(Err(e.into()));
            return;
          }
        };

        progress_reporter_.update("Converting XML to LogQuest format");

        if let Err(gina_import_error) =
          from_gina.convert_import(index, &import_time, &progress_reporter_)
        {
          progress_reporter_.update(format!(
            "LogQuest conversion failed!\nError: {}",
            gina_import_error.to_string()
          ));
        } else {
          progress_reporter_.update("LogQuest conversion complete!\nReloading data");
        }
        _ = tx_result.send(Ok(()));
      });
    })
    .expect("Could not spawn a thread to import a GINA file!"); // panic-worthy

  (progress_reporter, rx_progess_update, rx_result)
}
