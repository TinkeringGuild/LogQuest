use crate::{
  audio::AudioMixer,
  logs::{
    active_character_detection::{ActiveCharacterDetector, Character},
    log_event_broadcaster::LogEventBroadcaster,
    log_reader::LogReader,
    Line,
  },
  matchers::MatchContext,
  state::state_handle::StateHandle,
  triggers::{effects::TriggerEffect, TriggerGroup, TriggerGroupDescendant},
  tts::TTS,
};
use std::collections::LinkedList;
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::oneshot};
use tracing::{debug, error, info, warn};

const REACTOR_EVENT_QUEUE_DEPTH: usize = 1000;

struct EventLoop {
  log_events: LogEventBroadcaster,
  reactor_tx: mpsc::Sender<ReactorEvent>,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  mixer: AudioMixer,
  t2s_tx: mpsc::Sender<TTS>,
  state: StateHandle,
}

#[derive(Debug)]
enum ReactorEvent {
  SetActiveCharacter(Option<Character>),
  ExecTriggerEffect {
    effect: TriggerEffect,
    context: Arc<MatchContext>,
  },
  Shutdown,
}

#[derive(thiserror::Error, Debug)]
pub enum ReactorStartError {
  /// This error is recoverable
  #[error("Cannot start reactor when no Logs directory is known")]
  NoLogsDir,
  #[error("Failed to watch filesystem events for Logs directory")]
  WatchError(#[from] notify::Error),
}

type StopReactor = Box<dyn FnOnce() + 'static + Send + Sync>;

pub fn start_when_config_is_ready(
  state_handle: &StateHandle,
) -> oneshot::Receiver<Result<StopReactor, ReactorStartError>> {
  let state_handle = state_handle.to_owned();
  let (resolver, future) = oneshot::channel::<Result<StopReactor, ReactorStartError>>();
  spawn(async move {
    loop {
      match start_if_ready(&state_handle) {
        Ok(Some(stop_reactor)) => {
          info!("The EverQuest directory has been set properly. Reactor starting...");
          let _ = resolver.send(Ok(stop_reactor));
          break;
        }
        Err(e) => {
          error!("Waited until config was ready to start reactor, but encountered an error when starting it");
          let _ = resolver.send(Err(e));
          break;
        }
        Ok(None) => {}
      }

      warn!("The EverQuest directory is not set in the config! Set the directory to start the LogQuest reactor");
      state_handle.config_updated.notified().await;
    }
  });
  future
}

fn start_if_ready(state: &StateHandle) -> Result<Option<StopReactor>, ReactorStartError> {
  // LogQuestConfig validates that the directory saved is a valid EQ dir before
  // it allows a value to be set into `everquest_directory`
  let is_ready = state.select_config(|c| c.is_ready());
  if is_ready {
    match start(state.clone()) {
      Ok(stop_reactor) => Ok(Some(stop_reactor)),
      Err(err) => Err(err),
    }
  } else {
    Ok(None)
  }
}

pub fn start(state_handle: StateHandle) -> Result<StopReactor, ReactorStartError> {
  let Some(logs_dir) = state_handle.select_config(|config| config.logs_dir_path.clone()) else {
    return Err(ReactorStartError::NoLogsDir);
  };
  let log_events = LogEventBroadcaster::new(&logs_dir)?;
  let active_detector = ActiveCharacterDetector::start(log_events.subscribe());
  let (reactor_tx, reactor_rx) = mpsc::channel::<ReactorEvent>(REACTOR_EVENT_QUEUE_DEPTH);

  let trigger_count = state_handle.select_triggers(|root| root.trigger_count());
  debug!("Starting reactor with {trigger_count} triggers");

  let reactor_tx_ = reactor_tx.clone();
  let _ = spawn(react_to_active_character_change(
    state_handle.clone(),
    active_detector,
    reactor_tx_,
  ));

  let join_handle = spawn(run_event_loop(
    log_events,
    reactor_tx.clone(),
    reactor_rx,
    state_handle,
  ));

  let reactor_tx_ = reactor_tx.clone();
  let stopper = move || {
    let rt = tauri::async_runtime::handle();
    let _ = reactor_tx_.blocking_send(ReactorEvent::Shutdown);
    if let Err(e) = rt.block_on(join_handle) {
      error!("Error blocking on the reactor shutdown join handle! {e:?}");
    }
  };

  Ok(Box::new(stopper))
}

async fn run_event_loop(
  log_events: LogEventBroadcaster,
  reactor_tx: mpsc::Sender<ReactorEvent>,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  state: StateHandle,
) {
  let event_loop = EventLoop::new(log_events, reactor_tx, reactor_rx, state);
  // TODO: Create a startup sound to test the audio playback.
  event_loop.run().await;
}

impl EventLoop {
  fn new(
    log_events: LogEventBroadcaster,
    reactor_tx: mpsc::Sender<ReactorEvent>,
    reactor_rx: mpsc::Receiver<ReactorEvent>,
    state: StateHandle,
  ) -> Self {
    let t2s_tx = create_tts_engine(state.clone());
    let mixer = AudioMixer::new();
    Self {
      state,
      log_events,
      reactor_tx,
      reactor_rx,
      t2s_tx,
      mixer,
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

    let mut log_reader: LogReader = LogReader::idle();

    // When there is no active LogReader, a temporary broadcast::Receiver<Line> must be
    // created that keeps the select loop working. If a Receiver's Sender is dropped,
    // the channel closes, so the Sender must be kept around together. When a LogReader
    // is started, it maintains ownership of its own Sender, so no Sender needs to be
    // kept around, hence why there is an Option wrapping the Sender in this tuple.
    let mut line_chan: (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) =
      idle_line_chan();

    loop {
      debug!("TICK...");
      select! {
        reactor_event = self.reactor_rx.recv() => {
          debug!("GOT REACTOR EVENT: {reactor_event:?}");
          match reactor_event {
            None => break,
            Some(ReactorEvent::Shutdown) => {
              debug!("Reactor shutting down");
              break;
            }
            Some(ReactorEvent::SetActiveCharacter(Some(new_char))) => {
              let new_log_reader = LogReader::start(&new_char.log_file_pathbuf(), self.log_events.subscribe());
              line_chan = (None, new_log_reader.subscribe());
              log_reader = new_log_reader;
              info!("Setting reactor state for new current character: {new_char:?}");
              self.state.update_reactor(|r| r.current_character = Some(new_char));
            }
            Some(ReactorEvent::SetActiveCharacter(None)) => {
              log_reader.stop();
              log_reader = LogReader::idle();
              line_chan = idle_line_chan();
              info!("Setting reactor state to have no current character");
              self.state.update_reactor(|r| r.current_character = None);
            }
            Some(ReactorEvent::ExecTriggerEffect{effect, context}) => {
              self.exec_effect(&effect, &context).await;
            }
          }
        }
        line = line_chan.1.recv() => {
          match line {
            Ok(line) => {
              debug!("LINE: {:?}", line);
              self.react_to_line(&line).await;
            }
            Err(_recv_error) => {
              // LINE_CHAN IS CLOSED! NEED TO DE-DUPLICATE LOOP RESET LOGIC
            }
          }
        }
      }
    }
    let _ = self.log_events.stop();
    debug!("Event Loop finished");
  }

  async fn react_to_line(&self, line: &Line) {
    self.state.with_reactor(|reactor_state| {
      self.state.with_triggers(|root| {
        let mut next_groups: LinkedList<&TriggerGroup> = LinkedList::from_iter(root.iter());

        let Some(character) = &reactor_state.current_character else {
          warn!("Cannot process line! No current character detected!");
          return;
        };

        while let Some(dequeued_tg) = next_groups.pop_front() {
          for tgd in dequeued_tg.children.iter() {
            match tgd {
              TriggerGroupDescendant::T(trigger) => {
                if !trigger.enabled {
                  continue;
                }
                if let Some(match_context) = trigger.filter.check(&line.content, &character.name) {
                  let match_context = Arc::new(match_context);
                  for effect in trigger.effects.iter() {
                    debug!("TRIGGER EFFECT: {effect:?}");
                    self.send(ReactorEvent::ExecTriggerEffect {
                      effect: effect.clone(),
                      context: match_context.clone(),
                    });
                  }
                }
              }
              TriggerGroupDescendant::TG(tg) => {
                next_groups.push_back(tg);
              }
            }
          }
        }
      });
    });
  }

  fn send(&self, event: ReactorEvent) {
    let tx = self.reactor_tx.clone();
    spawn(async move {
      if let Err(_) = tx.send(event).await {
        error!("Tried to send a message to the Reactor but its channel is closed!");
      }
    });
  }

  async fn exec_effect(&self, effect: &TriggerEffect, context: &MatchContext) {
    match effect {
      TriggerEffect::PlayAudioFile(Some(file_path)) => {
        if let Err(e) = self.mixer.play_file(&file_path.render(&context)) {
          error!("Error playing file! {e:?}");
        }
      }
      TriggerEffect::TextToSpeech(template) => {
        let message = template.render(&context);
        if let Err(_) = self
          .t2s_tx
          .send(TTS::Speak {
            text: message.clone(),
            interrupt: false,
          })
          .await
        {
          error!(r#"Text-to-Speech channel closed! Ignoring TTS message: "{message}""#);
        }
      }
      _ => {}
    }
  }
}

async fn react_to_active_character_change(
  state_handle: StateHandle,
  mut active_character_detector: ActiveCharacterDetector,
  tx: mpsc::Sender<ReactorEvent>,
) {
  debug!("Initializing reactor active character change detector");
  loop {
    select! {
      _signal = active_character_detector.changed() => {
        let new_current_char = active_character_detector.current();
        if let Err(mpsc::error::SendError(_)) = tx.send(ReactorEvent::SetActiveCharacter(new_current_char.clone())).await {
          break;
        }
        state_handle.update_reactor(|state| {
          state.current_character = new_current_char;
        });
      }
    }
  }
  info!("Active character change detector stopping");
  active_character_detector.stop();
}

fn create_tts_engine(state: StateHandle) -> mpsc::Sender<TTS> {
  let (tx, rx) = mpsc::channel::<TTS>(100);
  if let Err(e) = crate::tts::spawn(state, rx) {
    // If TTS does not initialize, the receiver will be closed, so attempts to send messages will
    // result in a send error. This is an acceptable failure mode. Errors are printed if the
    // messages get dropped.
    error!("Could not initialize Text-to-Speech engine! Feature will be disabled. Error: {e:?}");
  }
  tx
}

fn idle_line_chan() -> (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) {
  let chan = broadcast::channel::<Line>(1);
  (Some(chan.0), chan.1)
}
