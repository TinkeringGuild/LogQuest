//! This file contains only code that is available in debug builds of LogQuest.
#![cfg(debug_assertions)]

use crate::{
  cli,
  commands::Bootstrap,
  common::{fatal_error, fatal_if_err, timestamp::Timestamp, UUID},
  gina::xml::load_gina_triggers_from_file_path,
  logs::{
    log_event_broadcaster::{LogEventBroadcaster, NotifyError},
    log_reader::LogReader,
  },
  matchers,
  triggers::{effects::TriggerEffect, Trigger, TriggerGroup, TriggerGroupDescendant},
};
use std::fs;
use std::path::PathBuf;
use tracing::info;
use ts_rs::TS;

pub fn test_trigger_group() -> TriggerGroup {
  fn re(s: &str) -> matchers::Matcher {
    matchers::Matcher::GINA(s.try_into().unwrap())
  }

  let trigger = Trigger {
    id: UUID::new(),
    name: "Tells / Hail".to_owned(),
    comment: None,
    created_at: Timestamp::now(),
    updated_at: Timestamp::now(),
    enabled: true,
    filter: vec![
      re(r"^([A-Za-z]+) -> {C}: (.+)$"),
      re(r"^([A-Za-z]+) says, 'Hail, {C}'$"),
    ]
    .into(),
    effects: vec![
      TriggerEffect::TextToSpeech("Hail, ${C}!".into()),
      // TriggerEffect::PlayAudioFile(Some(
      //   "/home/j/Downloads/sound effects/hail/hail-exclaim-callum.ogg".into(),
      // )),
      // TriggerEffect::OverlayMessage("💬${1}: ${2}".into()),
    ],
  };

  TriggerGroup {
    id: UUID::new(),
    name: "Test".to_owned(),
    children: vec![TriggerGroupDescendant::T(trigger)],
    comment: None,
    created_at: Timestamp::now(),
    updated_at: Timestamp::now(),
  }
}

/// This is mainly useful for debugging filesystem events
pub fn tail(log_file_path: &std::path::Path) -> Result<(), NotifyError> {
  info!("Watch log file events for {}", log_file_path.display());
  let rt = tokio::runtime::Runtime::new().unwrap();
  let mut fs_events = LogEventBroadcaster::new(&log_file_path)?;
  fs_events.start()?;
  let fs_event_rx = fs_events.subscribe();

  let reader = LogReader::start(&log_file_path, fs_event_rx);
  let mut rx = reader.subscribe();

  // info!("Spawning task");
  // rt.spawn(async move {
  //   let sleep_secs = 30;
  //   info!("Sleeping for {sleep_secs} seconds, then stopping the log reader");
  //   tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
  //   info!("Stopping log reader now");
  //   reader.stop();
  // });

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

/// This function is designed to be called from the CLI so it exits on error
pub fn convert_gina(path: &PathBuf, format: cli::ConvertGinaFormat, out: Option<PathBuf>) {
  let mut writer: Box<dyn std::io::Write> = if let Some(out_path) = out {
    Box::new(fatal_if_err(fs::File::create(out_path)))
  } else {
    Box::new(std::io::stdout())
  };

  let from_gina = fatal_if_err(load_gina_triggers_from_file_path(path));

  match format {
    cli::ConvertGinaFormat::GinaInternal => {
      fatal_if_err(writeln!(writer, "{from_gina:#?}"));
    }
    cli::ConvertGinaFormat::GinaJSON => {
      let pretty_json = fatal_if_err(serde_json::to_string_pretty(&from_gina));
      fatal_if_err(writeln!(writer, "{pretty_json}"));
    }
    _ => {}
  }

  let root_trigger_group = fatal_if_err(from_gina.to_lq(&Timestamp::now()));
  match format {
    cli::ConvertGinaFormat::Internal => {
      fatal_if_err(writeln!(writer, "{root_trigger_group:#?}"));
    }
    cli::ConvertGinaFormat::JSON => {
      let pretty_json = fatal_if_err(serde_json::to_string_pretty(&root_trigger_group));
      fatal_if_err(writeln!(writer, "{pretty_json}"));
    }
    _ => unreachable!(/* all four cases are handled by the two match expressions */),
  }
}

pub fn generate_typescript() -> Result<(), ts_rs::ExportError> {
  let out_dir = generated_typescript_dir();
  Bootstrap::export_all_to(&out_dir)?;
  info!("Exported TypeScript files to {}", out_dir.display());
  Ok(())
}

fn generated_typescript_dir() -> PathBuf {
  let ts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../src/generated/")
    .canonicalize()
    .expect("Could not canonicalize path to the generated TypeScript dir!");

  if !ts_dir.is_dir() {
    fatal_error("The src/generated/ dir does not exist!");
  }
  ts_dir
}
