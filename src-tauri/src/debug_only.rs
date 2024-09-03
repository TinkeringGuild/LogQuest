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
    log_file_cursor::LogFileCursor,
    log_line_stream::LogLineStream,
  },
  matchers::{self, MatchContext},
  reactor::EventLoop,
  state::timer_manager::{TimerCommand, TimerStateUpdate},
  triggers::{
    effects::{Effect, EffectWithID},
    timers::{Timer, TimerStartPolicy},
    Trigger, TriggerGroup, TriggerGroupDescendant,
  },
};
use std::{ffi::OsString, fs::File};
use std::{fs, sync::Arc};
use std::{
  path::{Path, PathBuf},
  process::Command,
};
use tauri::async_runtime::spawn;
use tokio_stream::StreamExt as _;
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
      EffectWithID::new(Effect::Speak {
        tmpl: "Hail, ${C}!".into(),
        interrupt: false,
      }),
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
pub fn generate_overlay_noise(event_loop: &EventLoop) {
  let context = event_loop.create_event_context(
    Arc::new(MatchContext::empty("Xenk")),
    Arc::new(LogFileCursor {
      path: String::new(),
      position: 0,
    }),
  );

  warn!("GENERATING OVERLAY MESSAGE NOISE");
  let context_ = context.clone();
  spawn(async move {
    loop {
      tokio::time::sleep(std::time::Duration::from_secs(5)).await;
      let message = format!("Now: {}", Timestamp::now().to_string());
      warn!("Sending generated overlay message: {message}");
      context_.overlay_manager.message(message);
    }
  });

  warn!("GENERATING TIMER NOISE");
  let trigger_id = UUID::new();
  let context_ = context.clone();
  spawn(async move {
    loop {
      let context = context_.clone();
      let timer_name = "Visions of Grandeur";
      warn!("GENERATING TIMER NOISE WITH NAME: {timer_name}");
      context
        .timer_manager
        .start_timer(
          Timer {
            name_tmpl: timer_name.into(),
            duration: common::duration::Duration(42 * 60 * 1000 / 100),
            repeats: false,
            start_policy:
              TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(
                timer_name.into(),
              ),
            trigger_id: trigger_id.clone(),
            tags: vec![],
            effects: vec![],
          },
          context.clone(),
        )
        .await;
      tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
  });

  let trigger_id = UUID::new();
  let context_ = context.clone();
  spawn(async move {
    let timer_name = "Divine Aura";
    loop {
      warn!("GENERATING TIMER NOISE WITH NAME: {timer_name}");
      let context = context_.clone();
      let timer_manager = &context.timer_manager;
      timer_manager
        .start_timer(
          Timer {
            name_tmpl: timer_name.into(),
            duration: common::duration::Duration(20 * 1000),
            repeats: false,
            start_policy:
              TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(
                timer_name.into(),
              ),
            trigger_id: trigger_id.clone(),
            tags: vec![],
            effects: vec![],
          },
          context.clone(),
        )
        .await;
      tokio::time::sleep(std::time::Duration::from_secs(23)).await;
    }
  });

  let trigger_id = UUID::new();
  let context_ = context.clone();
  spawn(async move {
    let timer_name = "Reset Every 3 sec and I have a really long name";
    loop {
      warn!("GENERATING TIMER NOISE WITH NAME: {timer_name}");
      let context = context_.clone();
      let timer_manager = &context.timer_manager;
      let id = timer_manager
        .start_timer(
          Timer {
            name_tmpl: timer_name.into(),
            duration: common::duration::Duration(10 * 1000),
            repeats: false,
            start_policy:
              TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(
                timer_name.into(),
              ),
            trigger_id: trigger_id.clone(),
            tags: vec![],
            effects: vec![],
          },
          context.clone(),
        )
        .await
        .unwrap();

      // let id_ = id.clone();
      // let timer_manager_ = timer_manager.clone();
      // spawn(async move {
      //   loop {
      //     let _ = timer_manager_
      //       .send(TimerCommand::SetHidden(id_.clone(), true))
      //       .await;
      //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      //     let _ = timer_manager_
      //       .send(TimerCommand::SetHidden(id_.clone(), false))
      //       .await;
      //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      //   }
      // });

      let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
      loop {
        interval.tick().await;
        let _ = timer_manager.send(TimerCommand::Restart(id.clone())).await;
      }
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

  let cursor = LogFileCursor::new(&log_file_path.to_string_lossy()).unwrap();

  rt.block_on(async move {
    let stream = LogLineStream::create(&cursor, fs_event_rx);
    let mut stream = stream.await.unwrap();
    while let Some(line) = stream.next().await {
      info!("{line:#?}");
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
  delete_files(&files_in_dir_with_extension(&out_dir, "ts"));

  Bootstrap::export_all_to(&out_dir)?;
  TimerStateUpdate::export_all_to(&out_dir)?;
  ProgressUpdate::export_all_to(&out_dir)?;

  #[allow(non_snake_case)]
  let LQ_VERSION: LogQuestVersion = LOG_QUEST_VERSION.clone();

  generate_typescript_constants_file(
    &out_dir,
    constants![
      LQ_VERSION,
      crate::commands::CROSS_DISPATCH_EVENT_NAME,
      crate::state::overlay::OVERLAY_EDITABLE_CHANGED_EVENT_NAME,
      crate::state::overlay::OVERLAY_MESSAGE_EVENT_NAME,
      crate::state::overlay::OVERLAY_STATE_UPDATE_EVENT_NAME,
      crate::state::state_tree::DEFAULT_OVERLAY_OPACITY,
      crate::ui::OVERLAY_WINDOW_LABEL,
      crate::ui::PROGRESS_UPDATE_EVENT_NAME,
      crate::ui::PROGRESS_UPDATE_FINISHED_EVENT_NAME
    ],
  );

  for ts_file in files_in_dir_with_extension(&out_dir, "ts").iter() {
    prettyify_typescript_file(ts_file);
  }

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

fn prettyify_typescript_file(path: &Path) {
  match Command::new("prettier").arg("--write").arg(path).status() {
    Ok(exit_status) => {
      if !exit_status.success() {
        fatal_error(format!(
          "`prettier` command failed with status code: {:?}",
          exit_status.code()
        ));
      }
    }
    Err(e) => {
      fatal_error(format!("Could not execute `prettier` command on TypeScript file. Is `prettier` installed? [ ERROR = {e:?} ]"));
    }
  }
}

fn files_in_dir_with_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
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
    .collect()
}

fn delete_files(files: &Vec<PathBuf>) {
  for file in files.iter() {
    fatal_if_err(fs::remove_file(file));
    info!("DELETED: {}", file.display());
  }
}
