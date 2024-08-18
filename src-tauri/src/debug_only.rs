//! This file contains only code that is available in debug builds of LogQuest.
#![cfg(debug_assertions)]

use crate::{
  cli,
  commands::Bootstrap,
  common::{
    self, fatal_error, fatal_if_err,
    progress_reporter::{ProgressReporter, ProgressUpdate},
    timestamp::Timestamp,
    LogQuestVersion, LOG_QUEST_VERSION, UUID,
  },
  gina::xml::load_gina_triggers_from_file_path,
  logs::{
    log_event_broadcaster::{LogEventBroadcaster, NotifyError},
    log_reader::LogReader,
  },
  matchers,
  state::timer_manager::{TimerManager, TimerStateUpdate},
  triggers::{
    effects::TriggerEffect, Timer, TimerStartPolicy, Trigger, TriggerGroup, TriggerGroupDescendant,
  },
};
use std::path::{Path, PathBuf};
use std::{ffi::OsString, fs::File};
use std::{fs, sync::Arc};
use tauri::async_runtime::spawn;
use tracing::{info, warn};
use ts_rs::TS;

/// This macro is used to generate a `constants.ts` file from Rust constants. It takes a list of
/// paths/identifiers, automatically JSON-serializes their values, and returns a vector of tuples
/// containing this data/metadata.
macro_rules! constants {
  ($($key:path),+) => {
    {
      let mut vec = Vec::new();
      $(
        let path = stringify!($key).to_owned();
        let name = path.rsplit("::").next().unwrap().to_owned();
        let value = serde_json::to_string(&$key).expect(&format!("Could not serialize {path}"));
        vec.push((path, name, value));
      )+
      vec
    }
  };
}

const CONSTANTS_TYPESCRIPT_FILENAME: &str = "constants.ts";

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

#[allow(unused)]
pub fn generate_timer_noise(timer_manager: Arc<TimerManager>) {
  warn!("GENERATING TIMER NOISE");

  let trigger_id = UUID::new();
  let timer_manager_ = timer_manager.clone();
  spawn(async move {
    let context = matchers::MatchContext::empty("Xenk");
    loop {
      let timer_name = "Visions of Grandeur";
      warn!("GENERATING TIMER NOISE WITH NAME: {timer_name}");
      timer_manager_
        .start_timer(
          &Timer {
            name: timer_name.into(),
            duration: common::duration::Duration(42 * 60 * 1000 / 100),
            repeats: false,
            start_policy: TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerHavingName(
              timer_name.into(),
            ),
            trigger_id: trigger_id.clone(),
            tags: vec![],
            updates: vec![],
          },
          &context,
        )
        .await;
      tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
  });

  let trigger_id = UUID::new();
  let timer_manager_ = timer_manager.clone();
  spawn(async move {
    let timer_name = "Divine Aura";
    let context = matchers::MatchContext::empty("Xenk");
    loop {
      warn!("GENERATING TIMER NOISE WITH NAME: {timer_name}");
      timer_manager_
        .start_timer(
          &Timer {
            name: timer_name.into(),
            duration: common::duration::Duration(20 * 1000),
            repeats: false,
            start_policy: TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerHavingName(
              timer_name.into(),
            ),
            trigger_id: trigger_id.clone(),
            tags: vec![],
            updates: vec![],
          },
          &context,
        )
        .await;
      tokio::time::sleep(std::time::Duration::from_secs(23)).await;
    }
  });
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

  let (progress, _) = ProgressReporter::new();
  let from_gina = fatal_if_err(load_gina_triggers_from_file_path(path, &progress));

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

  let root_trigger_group = fatal_if_err(from_gina.to_lq(&Timestamp::now(), &progress));
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
  delete_files_in_dir_with_extension(&out_dir, "ts");

  Bootstrap::export_all_to(&out_dir)?;
  TimerStateUpdate::export_all_to(&out_dir)?;
  ProgressUpdate::export_all_to(&out_dir)?;

  #[allow(non_snake_case)]
  let LQ_VERSION: LogQuestVersion = LOG_QUEST_VERSION.clone();

  generate_typescript_constants_file(
    &out_dir,
    constants![
      LQ_VERSION,
      crate::ui::PROGRESS_UPDATE_EVENT_NAME,
      crate::ui::PROGRESS_UPDATE_FINISHED_EVENT_NAME,
      crate::state::overlay::OVERLAY_MESSAGE_EVENT_NAME,
      crate::state::overlay::OVERLAY_STATE_UPDATE_EVENT_NAME,
      crate::state::overlay::OVERLAY_EDITABLE_CHANGED_EVENT_NAME
    ],
  );

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

fn generate_typescript_constants_file(out_dir: &Path, constants: Vec<(String, String, String)>) {
  use std::io::Write as _;

  let file_path = out_dir.join(CONSTANTS_TYPESCRIPT_FILENAME);
  let mut constants_file = fatal_if_err(File::create(&file_path));

  fatal_if_err(writeln!(
    &mut constants_file,
    "// GENERATED FILE - DO NOT EDIT\n"
  ));

  for (const_path, const_name, const_value) in constants.iter() {
    fatal_if_err(writeln!(&mut constants_file, "/// From `{const_path}`"));
    fatal_if_err(writeln!(
      &mut constants_file,
      "export const {const_name} = {const_value};\n"
    ));
  }

  fatal_if_err(constants_file.flush());

  info!(
    "Generated TypeScript constants file: {}",
    file_path.display()
  );
}

fn delete_files_in_dir_with_extension(dir: &Path, extension: &str) {
  let extension: OsString = extension.into();
  dir
    .read_dir()
    .expect("read_dir error")
    .filter_map(|dir_entry| {
      dir_entry
        .ok()
        .map(|entry| entry.path())
        .take_if(|path| path.extension() == Some(&extension))
    })
    .for_each(|f| {
      fatal_if_err(fs::remove_file(&f));
      info!("DELETED: {}", f.display());
    });
}
