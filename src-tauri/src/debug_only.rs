//! This file contains only code that is available in debug builds of LogQuest.
use crate::{
  cli,
  common::timestamp::Timestamp,
  common::UUID,
  gina::xml::load_gina_triggers_from_file_path,
  logs::log_event_broadcaster::LogEventBroadcaster,
  logs::log_reader::LogReader,
  state::config,
  {matchers, triggers},
};
use anyhow::bail;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};
use ts_rs::TS as _;

#[cfg(debug_assertions)]
pub fn test_trigger_group() -> triggers::TriggerGroup {
  let now = Timestamp::now;

  fn re(s: &str) -> matchers::Matcher {
    matchers::Matcher::GINA(s.try_into().unwrap())
  }

  let trigger = triggers::Trigger {
    id: UUID::new(),
    name: "Tells / Hail".to_owned(),
    comment: None,
    created_at: now(),
    updated_at: now(),
    enabled: true,
    filter: vec![
      re(r"^([A-Za-z]+) -> {C}: (.+)$"),
      re(r"^([A-Za-z]+) says, 'Hail, {C}'$"),
    ]
    .into(),
    effects: vec![
      triggers::TriggerEffect::TextToSpeech("Hail, ${C}!".into()),
      // triggers::TriggerEffect::PlayAudioFile(Some(
      //   "/home/j/Downloads/sound effects/hail/hail-exclaim-callum.ogg".into(),
      // )),
      // triggers::TriggerEffect::OverlayMessage("💬${1}: ${2}".into()),
    ],
  };

  triggers::TriggerGroup {
    id: UUID::new(),
    name: "Test".to_owned(),
    children: vec![triggers::TriggerGroupDescendant::T(trigger)],
    comment: None,
    created_at: now(),
    updated_at: now(),
  }
}

#[cfg(debug_assertions)]
pub fn tail(log_file_path: &std::path::Path) -> anyhow::Result<()> {
  info!("In tail");
  let rt = tokio::runtime::Runtime::new().unwrap();
  let mut fs_events = LogEventBroadcaster::new(&log_file_path)?;
  fs_events.start()?;
  let fs_event_rx = fs_events.subscribe();

  let reader = LogReader::start(&log_file_path, fs_event_rx);
  let mut rx = reader.subscribe();

  info!("Spawning task");
  rt.spawn(async move {
    let sleep_secs = 30;
    info!("Sleeping for {sleep_secs} seconds, then stopping the log reader");
    tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
    info!("Stopping log reader now");
    reader.stop();
  });

  rt.block_on(async move {
    loop {
      tokio::select! {
          line = rx.recv() => {
              info!("{line:#?}");
          }
      }
    }
  });
  Ok(())
}

#[cfg(debug_assertions)]
pub fn convert_gina(
  path: &PathBuf,
  format: cli::ConvertGinaFormat,
  out: Option<PathBuf>,
) -> anyhow::Result<()> {
  let mut writer: Box<dyn std::io::Write> = if let Some(out_path) = out {
    Box::new(fs::File::create(out_path)?)
  } else {
    Box::new(std::io::stdout())
  };

  let from_gina = load_gina_triggers_from_file_path(path)?;

  match format {
    cli::ConvertGinaFormat::GinaInternal => {
      writeln!(writer, "{from_gina:#?}")?;
      return Ok(());
    }
    cli::ConvertGinaFormat::GinaJSON => match serde_json::to_string_pretty(&from_gina) {
      Ok(pretty_json) => {
        writeln!(writer, "{pretty_json}")?;
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
      writeln!(writer, "{root_trigger_group:#?}")?;
    }
    cli::ConvertGinaFormat::JSON => match serde_json::to_string_pretty(&root_trigger_group) {
      Ok(pretty_json) => writeln!(writer, "{pretty_json}")?,
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
