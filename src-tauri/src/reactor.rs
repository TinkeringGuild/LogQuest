use crate::common::fatal_error;
use crate::common::timestamp::Timestamp;
use crate::gina::regex::RegexGINA;
use crate::logs::active_character_detection::{ActiveCharacterDetector, CharacterNameWithServer};
use crate::logs::log_event_broadcaster::LogEventBroadcaster;
use crate::logs::log_reader::LogReader;
use crate::logs::Line;
use crate::matchers;
use crate::triggers::{Trigger, TriggerEffect, TriggerGroup, TriggerGroupDescendant};
use std::path::Path;
use tokio::runtime::Handle;
use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info};

#[derive(Debug)]
enum ReactorEvent {
  SetActiveCharacter(Option<CharacterNameWithServer>),
}

pub fn spawn(rt: &Handle, logs_dir: &Path) -> anyhow::Result<()> {
  let log_events = LogEventBroadcaster::new(&logs_dir)?;
  let active_detector = ActiveCharacterDetector::start(rt, log_events.subscribe());
  let (reactor_tx, reactor_rx) = mpsc::channel::<ReactorEvent>(256);
  let _ = rt.spawn(react_to_active_character_change(
    active_detector,
    reactor_tx,
  ));
  let join_handle = rt.spawn(event_loop(rt.clone(), log_events, reactor_rx));
  if let Err(e) = rt.block_on(join_handle) {
    fatal_error(&e.to_string());
  }
  Ok(())
}

// TODO: At the moment, because filesystem events are used to begin reading a log file, the very
// first line(s) appended to a log may get missed because the log wasn't being watched. To fix this
// the system will have to keep state of the size of all log files, then seek to the appropriate point
// when starting to read from the file. This could be implemented with some kind of LogEventCoordinator
// but it would need to atomically queue up new messages when it changes the active LogReader, and the
// logic here for reading lines might need to take into consideration that some lines received are
// stale. This could possibly be solved by implementing a recv() method on LogEventCoordinator that
// synchronously updates which underlying recv() future is returned by its own recv() method.
// Currently the code assumes that active character detection is enabled, so to support multiple
// concurrent overlays, the logic might become considerably more complex.
async fn event_loop(
  rt: Handle,
  mut log_event_broadcaster: LogEventBroadcaster,
  mut reactor_rx: mpsc::Receiver<ReactorEvent>,
) {
  log_event_broadcaster
    .start()
    .expect("COULD NOT START LOG EVENT BROADCASTER");

  debug!("Initializing reactor event loop");

  let mut current_character: Option<CharacterNameWithServer> = None;
  let mut log_reader: LogReader = LogReader::idle();

  // When there is no active LogReader, a temporary broadcast::Receiver<Line> must be
  // created that keeps the select loop working. If a Receiver's Sender is dropped,
  // the channel closes, so the Sender must be kept around together. When a LogReader
  // is started, it maintains ownership of its own Sender, so no Sender needs to be
  // kept around, hence why there is an Option wrapping the Sender in this tuple.
  let mut line_chan: (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) =
    idle_line_chan();

  let tg = test_trigger_group();
  loop {
    select! {
      reactor_event = reactor_rx.recv() => {
        debug!("GOT REACTOR EVENT: {reactor_event:?}");
        match reactor_event {
          None => break,
          Some(ReactorEvent::SetActiveCharacter(Some(new_char))) => {
            let new_log_reader = LogReader::start(rt.clone(), &new_char.log_file_pathbuf(), log_event_broadcaster.subscribe());
            line_chan = (None, new_log_reader.subscribe());
            log_reader = new_log_reader;
            current_character = Some(new_char);
          }
          Some(ReactorEvent::SetActiveCharacter(None)) => {
            log_reader.stop();
            log_reader = LogReader::idle();
            line_chan = idle_line_chan();
            current_character = None;
          }
        }
      }
      line = line_chan.1.recv() => {
        if let Some(CharacterNameWithServer { name, .. }) = &current_character {
          match line {
            Ok(line) => {
              debug!("LINE: {:?}", line);
              react_to_line(&line, &vec![&tg], name);
            }
            Err(_recv_error) => {
              // LINE_CHAN IS CLOSED! NEED TO DE-DUPLICATE LOOP RESET LOGIC
            }
          }

        }
      }
    }
  }
  let _ = log_event_broadcaster.stop();
  debug!("Event Loop finished");
}

fn react_to_line(line: &Line, trigger_groups: &Vec<&TriggerGroup>, char_name: &str) {
  for tg in trigger_groups.iter() {
    react_to_line_with_trigger_group(line, tg, char_name);
  }
}

fn react_to_line_with_trigger_group(line: &Line, trigger_group: &TriggerGroup, char_name: &str) {
  for tgd in trigger_group.children.iter() {
    match tgd {
      TriggerGroupDescendant::T(trigger) => {
        react_to_line_with_trigger(line, trigger, char_name);
      }
      TriggerGroupDescendant::TG(group) => react_to_line_with_trigger_group(line, group, char_name),
    }
  }
}

fn react_to_line_with_trigger(line: &Line, trigger: &Trigger, char_name: &str) {
  if let Some(_captures) = trigger.captures(&line.content, char_name) {
    for effect in trigger.effects.iter() {
      debug!("TRIGGER EFFECT: {effect:?}");
    }
  }
}

async fn react_to_active_character_change(
  mut active_character_detector: ActiveCharacterDetector,
  tx: mpsc::Sender<ReactorEvent>,
) {
  debug!("Initializing reactor active character change detector");
  loop {
    select! {
      _signal = active_character_detector.changed() => {
        let new_current_char = active_character_detector.current();
        info!("{:#?}", new_current_char);
        match tx.send(ReactorEvent::SetActiveCharacter(new_current_char)).await {
          Err(mpsc::error::SendError(_)) =>  break,
          _ => {}
        }
      }
    }
  }
  active_character_detector.stop();
}

fn test_trigger_group() -> TriggerGroup {
  let now = Timestamp::now;

  fn re(s: &str) -> matchers::Matcher {
    matchers::Matcher::GINA(RegexGINA::from_str(s).unwrap())
  }

  let trigger = Trigger {
    name: "Tells / Hail".to_owned(),
    comment: None,
    created_at: now(),
    updated_at: now(),
    enabled: true,
    filter: vec![
      re(r"^([A-Za-z]+) -> {C}: (.+)$"),
      re(r"^([A-Za-z]+) says, 'Hail, {C}'$"),
    ],
    effects: vec![
      TriggerEffect::PlayAudioFile(Some(
        "/home/j/Downloads/sound effects/hail/hail-exclaim-callum.mp3".into(),
      )),
      TriggerEffect::OverlayMessage("💬{1}: {2}".into()),
    ],
  };

  TriggerGroup {
    name: "Test".to_owned(),
    children: vec![TriggerGroupDescendant::T(trigger)],
    comment: None,
    created_at: now(),
    updated_at: now(),
  }
}

fn idle_line_chan() -> (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) {
  let chan = broadcast::channel::<Line>(1);
  (Some(chan.0), chan.1)
}
