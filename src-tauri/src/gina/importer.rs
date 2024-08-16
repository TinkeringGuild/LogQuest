use super::{
  conversion::GINAConversionError,
  xml::{load_gina_triggers_from_file_path, GINAParseError},
  GINATriggers,
};
use crate::{
  common::{
    progress_reporter::{ProgressReporter, ProgressUpdate},
    timestamp::Timestamp,
  },
  triggers::TriggerGroup,
};
use serde::{Deserialize, Serialize};
use std::thread;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};
use tokio::sync::{oneshot, watch};

#[derive(Debug, Serialize, Deserialize)]
pub struct GINAImport {
  file_path: PathBuf,
  import_time: Timestamp,
  from_gina: GINATriggers,
  pub converted: Vec<TriggerGroup>,
}

#[derive(thiserror::Error, Debug)]
pub enum GINAImportError {
  #[error("GINA conversion error")]
  ConversionError(#[from] GINAConversionError),
  #[error("GINA parse error")]
  ParseError(#[from] GINAParseError),
}

// TODO: This should aggregate all tags and create an index
impl GINAImport {
  pub fn load(
    file_path: &Path,
  ) -> (
    Arc<ProgressReporter>,
    watch::Receiver<ProgressUpdate>,
    oneshot::Receiver<Result<Self, GINAImportError>>,
  ) {
    let (progress_reporter, rx_progess_update) = ProgressReporter::new();
    let progress_reporter = Arc::new(progress_reporter);

    let (tx_result, rx_result) = oneshot::channel::<Result<Self, GINAImportError>>();

    let file_path = file_path.to_owned();
    let progress_reporter_ = progress_reporter.clone();
    thread::spawn(move || {
      let imported = import(file_path, &progress_reporter_);
      progress_reporter_.update("LogQuest conversion complete!\nSending data");
      let _ = tx_result.send(imported);
    });

    (progress_reporter, rx_progess_update, rx_result)
  }
}

fn import(
  file_path: PathBuf,
  progress: &Arc<ProgressReporter>,
) -> Result<GINAImport, GINAImportError> {
  let import_time: Timestamp = Timestamp::now();
  progress.update("Parsing GINA XML");
  let from_gina = load_gina_triggers_from_file_path(&file_path, progress)?;

  progress.update("Converting XML to LogQuest format");
  let converted = from_gina.to_lq(&import_time, progress)?;

  let import = GINAImport {
    file_path: file_path.to_owned(),
    import_time,
    from_gina,
    converted,
  };
  Ok(import)
}
