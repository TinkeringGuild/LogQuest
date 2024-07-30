use crate::audio::AudioMixer;
use crate::logs::active_character_detection::{ActiveCharacterDetector, Character};
use crate::logs::log_event_broadcaster::LogEventBroadcaster;
use crate::logs::log_reader::LogReader;
use crate::logs::Line;
use crate::triggers::{Trigger, TriggerEffect, TriggerGroup, TriggerGroupDescendant};
use std::collections::LinkedList;
use std::path::Path;
use tauri::async_runtime::spawn;
use tauri::async_runtime::JoinHandle;
use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

#[derive(Debug)]
enum ReactorEvent {
  SetActiveCharacter(Option<Character>),
}

pub fn start(logs_dir: &Path) -> anyhow::Result<JoinHandle<()>> {
  let log_events = LogEventBroadcaster::new(&logs_dir)?;
  let active_detector = ActiveCharacterDetector::start(log_events.subscribe());
  let (reactor_tx, reactor_rx) = mpsc::channel::<ReactorEvent>(256);
  let _ = spawn(react_to_active_character_change(
    active_detector,
    reactor_tx,
  ));
  let join_handle = spawn(run_event_loop(log_events, reactor_rx));
  Ok(join_handle)
}

async fn run_event_loop(log_events: LogEventBroadcaster, reactor_rx: mpsc::Receiver<ReactorEvent>) {
  let event_loop = EventLoop::new(log_events, reactor_rx);
  event_loop.run().await;
}

struct EventLoop {
  log_events: LogEventBroadcaster,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  mixer: AudioMixer,
}
impl EventLoop {
  fn new(log_events: LogEventBroadcaster, reactor_rx: mpsc::Receiver<ReactorEvent>) -> Self {
    Self {
      log_events,
      reactor_rx,
      mixer: AudioMixer::new(),
    }
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
  async fn run(mut self) {
    self
      .log_events
      .start()
      .expect("COULD NOT START LOG EVENT BROADCASTER");

    debug!("Initializing reactor event loop");

    let mut current_character = None::<Character>;

    let mut log_reader: LogReader = LogReader::idle();

    // When there is no active LogReader, a temporary broadcast::Receiver<Line> must be
    // created that keeps the select loop working. If a Receiver's Sender is dropped,
    // the channel closes, so the Sender must be kept around together. When a LogReader
    // is started, it maintains ownership of its own Sender, so no Sender needs to be
    // kept around, hence why there is an Option wrapping the Sender in this tuple.
    let mut line_chan: (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) =
      idle_line_chan();

    let tg = crate::debug_only::test_trigger_group();

    loop {
      select! {
        reactor_event = self.reactor_rx.recv() => {
          debug!("GOT REACTOR EVENT: {reactor_event:?}");
          match reactor_event {
            None => break,
            Some(ReactorEvent::SetActiveCharacter(Some(new_char))) => {
              let new_log_reader = LogReader::start(&new_char.log_file_pathbuf(), self.log_events.subscribe());
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
          if let Some(Character { name, .. }) = &current_character {
            match line {
              Ok(line) => {
                debug!("LINE: {:?}", line);
                self.react_to_line(&line, &vec![&tg], &name).await;
              }
              Err(_recv_error) => {
                // LINE_CHAN IS CLOSED! NEED TO DE-DUPLICATE LOOP RESET LOGIC
              }
            }

          }
        }
      }
    }
    let _ = self.log_events.stop();
    debug!("Event Loop finished");
  }

  async fn react_to_line(&self, line: &Line, trigger_groups: &Vec<&TriggerGroup>, char_name: &str) {
    for tg in trigger_groups.iter() {
      self
        .react_to_line_with_trigger_group(line, tg, char_name)
        .await;
    }
  }

  // Async functions in Rust cannot be recursive (at least not without some serious complexity).
  // This function descends the Trigger/TriggerGroup tree in a different way that
  async fn react_to_line_with_trigger_group(
    &self,
    line: &Line,
    trigger_group: &TriggerGroup,
    char_name: &str,
  ) {
    let mut queue = LinkedList::from([trigger_group]);
    // NOTE!
    while let Some(dequeued_tg) = queue.pop_front() {
      for tgd in dequeued_tg.children.iter() {
        match tgd {
          TriggerGroupDescendant::T(trigger) => {
            self
              .react_to_line_with_trigger(line, trigger, char_name)
              .await;
          }
          TriggerGroupDescendant::TG(tg) => {
            queue.push_back(tg);
          }
        }
      }
    }
  }

  async fn react_to_line_with_trigger(&self, line: &Line, trigger: &Trigger, char_name: &str) {
    if let Some(_captures) = trigger.captures(&line.content, char_name) {
      for effect in trigger.effects.iter() {
        debug!("TRIGGER EFFECT: {effect:?}");
        self.exec_effect(effect, char_name).await;
      }
    }
  }

  async fn exec_effect(&self, effect: &TriggerEffect, char_name: &str) {
    match effect {
      TriggerEffect::PlayAudioFile(Some(file_path)) => {
        if let Err(e) = self.mixer.play_file(&file_path.render(char_name)) {
          error!("Error playing file! {e:?}");
        }
      }
      _ => {}
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

fn idle_line_chan() -> (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) {
  let chan = broadcast::channel::<Line>(1);
  (Some(chan.0), chan.1)
}
