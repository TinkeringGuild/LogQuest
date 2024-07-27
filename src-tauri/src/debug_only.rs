use crate::cli;
use crate::config;
use crate::gina::xml::load_gina_triggers_from_file_path;
use anyhow::bail;
use std::{fs, path::PathBuf};
use tracing::error;
use tracing::info;
use ts_rs::TS as _;

#[cfg(debug_assertions)]
pub fn convert_gina(path: &PathBuf, format: cli::ConvertGinaFormat) -> anyhow::Result<()> {
  use crate::common::timestamp::Timestamp;

  let from_gina = load_gina_triggers_from_file_path(path)?;
  match format {
    cli::ConvertGinaFormat::GinaInternal => {
      println!("{from_gina:#?}");
      return Ok(());
    }
    cli::ConvertGinaFormat::GinaJSON => match serde_json::to_string_pretty(&from_gina) {
      Ok(raw_json) => {
        println!("{raw_json}");
        return Ok(());
      }
      Err(e) => {
        error!("Failed to serialize GINA types to JSON!");
        bail!(e)
      }
    },
    _ => {}
  }

  let root_trigger_group = from_gina.to_lq(&Timestamp::now())?;
  match format {
    cli::ConvertGinaFormat::Internal => {
      println!("{root_trigger_group:#?}");
    }
    cli::ConvertGinaFormat::JSON => match serde_json::to_string_pretty(&root_trigger_group) {
      Ok(raw_json) => println!("{raw_json}"),
      Err(e) => {
        error!("Failed to serialize to JSON!");
        return Err(e.into());
      }
    },
    _ => unreachable!(/* all four cases are handled by the two match expressions */),
  }
  Ok(())
}

#[cfg(debug_assertions)]
pub fn generate_typescript() -> anyhow::Result<()> {
  let rust_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let out_dir = rust_dir.join("../src/generated/").canonicalize()?;
  if !out_dir.exists() {
    panic!("The src/generated/ dir does not exist!");
  }
  let out_file = out_dir.join("LogQuestConfig.ts");
  if out_file.exists() {
    info!("Deleting possibly stale file {}", out_file.display());
    if let Err(e) = fs::remove_file(&out_file) {
      panic!(
        "Could not delete the file {} [ {:#?} ]",
        out_file.display(),
        e
      );
    }
  }
  if let Err(e) = config::LogQuestConfig::export_all_to(&out_dir) {
    panic!("Could not export TypeScript! {:#?}", e);
  }

  info!("Exported LogQuestConfig to {}", out_file.display());

  Ok(())
}
