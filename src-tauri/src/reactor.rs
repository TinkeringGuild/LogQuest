use crate::{
  audio::AudioMixer,
  common::shutdown::quitter,
  logs::{
    active_character_detection::{ActiveCharacterDetector, Character},
    log_event_broadcaster::{LogEventBroadcaster, NotifyError},
    log_reader::LogReader,
    Line,
  },
  matchers::MatchContext,
  state::{overlay::OverlayManager, state_handle::StateHandle, timer_manager::TimerManager},
  triggers::{effects::Effect, TriggerGroup, TriggerGroupDescendant},
  tts::TTS,
};
use std::collections::LinkedList;
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::oneshot};
use tracing::{debug, debug_span, error, info, warn};

const REACTOR_EVENT_QUEUE_DEPTH: usize = 1000;

pub struct ReactorContext {
  pub timer_manager: Arc<TimerManager>,
  pub overlay_manager: Arc<OverlayManager>,
  pub mixer: Arc<AudioMixer>,
  pub t2s_tx: mpsc::Sender<TTS>,
  pub match_context: MatchContext,
}

#[derive(Debug)]
pub enum ReactorEvent {
  SetActiveCharacter(Option<Character>),
  ExecTriggerEffect {
    effect: Effect,
    context: Arc<MatchContext>,
  },
}

struct EventLoop {
  state: StateHandle,
  log_events: LogEventBroadcaster,
  reactor_tx: mpsc::Sender<ReactorEvent>,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  mixer: Arc<AudioMixer>,
  t2s_tx: mpsc::Sender<TTS>,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
}

#[derive(thiserror::Error, Debug)]
pub enum ReactorStartError {
  /// This error could be recoverable
  #[error("Cannot start reactor when no Logs directory is known")]
  NoLogsDir,
  #[error("Failed to watch filesystem events for Logs directory")]
  WatchError(#[from] NotifyError),
}

pub fn start_when_config_is_ready(
  state_handle: &StateHandle,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
) -> oneshot::Receiver<Result<mpsc::Sender<ReactorEvent>, ReactorStartError>> {
  let state_handle = state_handle.to_owned();
  let (resolver, future) =
    oneshot::channel::<Result<mpsc::Sender<ReactorEvent>, ReactorStartError>>();
  spawn(async move {
    loop {
      match start_if_ready(&state_handle, timer_manager.clone(), &overlay_manager) {
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

fn start_if_ready(
  state: &StateHandle,
  timer_manager: Arc<TimerManager>,
  overlay_manager: &Arc<OverlayManager>,
) -> Result<Option<mpsc::Sender<ReactorEvent>>, ReactorStartError> {
  // LogQuestConfig validates that the directory saved is a valid EQ dir before
  // it allows a value to be set into `everquest_directory`
  let is_ready = state.select_config(|c| c.is_ready());
  if is_ready {
    match start(state.clone(), timer_manager, overlay_manager.clone()) {
      Ok(tx_reactor) => Ok(Some(tx_reactor)),
      Err(err) => Err(err),
    }
  } else {
    Ok(None)
  }
}

pub fn start(
  state_handle: StateHandle,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
) -> Result<mpsc::Sender<ReactorEvent>, ReactorStartError> {
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

  spawn(run_event_loop(
    state_handle,
    log_events,
    reactor_tx.clone(),
    reactor_rx,
    timer_manager,
    overlay_manager,
  ));

  Ok(reactor_tx)
}

async fn run_event_loop(
  state: StateHandle,
  log_events: LogEventBroadcaster,
  reactor_tx: mpsc::Sender<ReactorEvent>,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
) {
  let event_loop = EventLoop::new(
    state,
    log_events,
    reactor_tx,
    reactor_rx,
    timer_manager,
    overlay_manager,
  );
  // TODO: Create a startup sound to test the audio playback.
  event_loop.run().await;
}

impl EventLoop {
  fn new(
    state: StateHandle,
    log_events: LogEventBroadcaster,
    reactor_tx: mpsc::Sender<ReactorEvent>,
    reactor_rx: mpsc::Receiver<ReactorEvent>,
    timer_manager: Arc<TimerManager>,
    overlay_manager: Arc<OverlayManager>,
  ) -> Self {
    let t2s_tx = create_tts_engine(state.clone());
    let mixer = Arc::new(AudioMixer::new());
    Self {
      state,
      log_events,
      reactor_tx,
      reactor_rx,
      t2s_tx,
      mixer,
      timer_manager,
      overlay_manager,
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

    let mut quit = quitter();
    loop {
      debug!("TICK...");
      select! {
        _ = &mut quit => {
          debug!("Reactor QUITTING");
          break;
        }
        reactor_event = self.reactor_rx.recv() => {
          debug!("GOT REACTOR EVENT: {reactor_event:?}");
          match reactor_event {
            None => break,
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
              self.exec_effect(effect, &context).await;
            }
          }
        }
        line = line_chan.1.recv() => {
          match line {
            Ok(line) => {
              debug!("LINE: {:?}", line);
              self.react_to_line(&line).await; // TODO: can spawn be used here if self is an &Arc<Self>?
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

  fn create_reactor_context(&self, match_context: &MatchContext) -> Arc<ReactorContext> {
    Arc::new(ReactorContext {
      timer_manager: self.timer_manager.clone(),
      overlay_manager: self.overlay_manager.clone(),
      mixer: self.mixer.clone(),
      t2s_tx: self.t2s_tx.clone(),
      match_context: match_context.clone(),
    })
  }

  async fn exec_effect(&self, effect: Effect, match_context: &MatchContext) {
    let reactor_context = self.create_reactor_context(match_context);
    spawn(async move {
      if let Err(effect_error) = effect.ready().fire(reactor_context).await {
        error!("Encountered error executing Effect: {effect_error:?}");
      }
    });
  }
}

async fn react_to_active_character_change(
  state_handle: StateHandle,
  mut active_character_detector: ActiveCharacterDetector,
  tx: mpsc::Sender<ReactorEvent>,
) {
  let mut quit = quitter();
  debug!("Initializing reactor active character change detector");
  loop {
    select! {
      _ = &mut quit => {
        break;
      }
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

  let tx_ = tx.clone();
  spawn(async move {
    quitter().await;
    let _ = tx_.send(TTS::Quit).await;
  });

  tx
}

fn idle_line_chan() -> (Option<broadcast::Sender<Line>>, broadcast::Receiver<Line>) {
  let chan = broadcast::channel::<Line>(1);
  (Some(chan.0), chan.1)
}
