use crate::{
  audio::AudioMixer,
  common::{clipboard::ClipboardWriter, shutdown::quitter},
  logs::{
    active_character_detection::{ActiveCharacterDetector, Character},
    log_event_broadcaster::{LogEventBroadcaster, NotifyError},
    log_file_cursor::{LogFileCursor, LogFileCursorCache},
    log_line_stream::LogLineStream,
    Line, LogFileEvent,
  },
  matchers::MatchContext,
  state::{
    overlay::OverlayManager,
    state_handle::StateHandle,
    timer_manager::{TimerContext, TimerManager},
  },
  triggers::{effects::EffectWithID, Trigger},
  tts::TTS,
};
use futures::StreamExt as _;
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tokio::sync::{broadcast, mpsc};
use tokio::{select, sync::oneshot};
use tracing::{debug, error, info, warn};

const REACTOR_EVENT_QUEUE_DEPTH: usize = 1000;

#[derive(Debug, Clone)]
pub struct EventContext {
  pub timer_manager: Arc<TimerManager>,
  pub overlay_manager: Arc<OverlayManager>,
  pub mixer: Arc<AudioMixer>,
  pub reactor_tx: mpsc::Sender<ReactorEvent>,
  pub t2s_tx: mpsc::Sender<TTS>,
  pub match_context: Arc<MatchContext>,
  pub cursor_after: Arc<LogFileCursor>,
  pub timer_context: Option<TimerContext>,
  pub clipboard: ClipboardWriter,
  pub tx_log_file_events: broadcast::Sender<Result<LogFileEvent, NotifyError>>,
}

#[derive(Debug)]
pub enum ReactorEvent {
  SetActiveCharacter(Option<Character>),
  ExecEffect {
    effect: EffectWithID,
    event_context: Arc<EventContext>,
  },
  TestAudioFile(String),
}

pub struct EventLoop {
  state: StateHandle,
  cursors: LogFileCursorCache,
  log_events: LogEventBroadcaster,
  reactor_tx: mpsc::Sender<ReactorEvent>,
  reactor_rx: mpsc::Receiver<ReactorEvent>,
  mixer: Arc<AudioMixer>,
  t2s_tx: mpsc::Sender<TTS>,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
  clipboard: ClipboardWriter,
}

#[derive(thiserror::Error, Debug)]
pub enum ReactorStartError {
  /// This error could be recoverable
  #[error("Cannot start reactor when no Logs directory is known")]
  NoLogsDir,

  #[error("Failed to watch filesystem events for Logs directory")]
  WatchError(#[from] NotifyError),

  #[error(transparent)]
  LogsDirIOError(#[from] std::io::Error),
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
          _ = resolver.send(Ok(stop_reactor));
          break;
        }
        Err(e) => {
          error!("Waited until config was ready to start reactor, but encountered an error when starting it");
          _ = resolver.send(Err(e));
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
  state: StateHandle,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
) -> Result<mpsc::Sender<ReactorEvent>, ReactorStartError> {
  let Some(logs_dir) = state.select_config(|config| config.logs_dir_path.clone()) else {
    return Err(ReactorStartError::NoLogsDir);
  };
  let cursors = LogFileCursorCache::scan_dir(&logs_dir)
    .map_err(|io_err| ReactorStartError::LogsDirIOError(io_err))?;

  let log_events = LogEventBroadcaster::new(&logs_dir)?;
  let active_detector = ActiveCharacterDetector::start(log_events.subscribe());
  let (reactor_tx, reactor_rx) = mpsc::channel::<ReactorEvent>(REACTOR_EVENT_QUEUE_DEPTH);

  let trigger_count = state.select_triggers(|index| index.trigger_count());
  debug!("Starting reactor with {trigger_count} triggers");

  _ = spawn(react_to_active_character_change(
    state.clone(),
    active_detector,
    reactor_tx.clone(),
  ));

  let reactor_tx_ = reactor_tx.clone();

  spawn(async move {
    debug!("Creating EventLoop");
    EventLoop::new(
      state,
      cursors,
      log_events,
      reactor_tx,
      reactor_rx,
      timer_manager,
      overlay_manager,
    )
    .run()
    .await;
  });

  Ok(reactor_tx_)
}

impl EventLoop {
  fn new(
    state: StateHandle,
    cursors: LogFileCursorCache,
    log_events: LogEventBroadcaster,
    reactor_tx: mpsc::Sender<ReactorEvent>,
    reactor_rx: mpsc::Receiver<ReactorEvent>,
    timer_manager: Arc<TimerManager>,
    overlay_manager: Arc<OverlayManager>,
  ) -> Self {
    let t2s_tx = create_tts_engine(state.clone());
    let mixer = Arc::new(AudioMixer::new());
    let clipboard = ClipboardWriter::new();
    Self {
      state,
      cursors,
      log_events,
      reactor_tx,
      reactor_rx,
      t2s_tx,
      mixer,
      timer_manager,
      overlay_manager,
      clipboard,
    }
  }

  async fn run(mut self) {
    debug!("Reactor run() event loop");

    // crate::debug_only::generate_overlay_noise(&self);

    self
      .log_events
      .start()
      .expect("COULD NOT START LOG EVENT BROADCASTER");

    let mut line_stream_maybe = None::<LogLineStream>;

    let mut quit = quitter();
    loop {
      debug!("TICK...");
      select! {
        _ = &mut quit => {
          debug!("Reactor QUITTING");
          break;
        }
        reactor_event = self.reactor_rx.recv() => {
          match reactor_event {
            None => break,
            Some(ReactorEvent::SetActiveCharacter(Some(new_char))) => {
              let Ok(cursor) = self.cursors.get_cursor_and_mark_size_stale(&new_char.log_file_path) else {
                error!("IO error determining file size {} - Ignoring file", new_char.log_file_path);
                continue;
              };
              let Ok(line_stream) = LogLineStream::create(&cursor, self.log_events.subscribe()).await else {
                error!("IO error trying to create LogLineStream for {}", new_char.log_file_path);
                continue;
              };

              line_stream_maybe = Some(line_stream);

              info!("Setting new current character in reactor state: {new_char:?}");
              self.state.update_reactor(|r| r.current_character = Some(new_char));
            }
            Some(ReactorEvent::SetActiveCharacter(None)) => {
              if let Some(line_stream) = line_stream_maybe.take() {
                self.cursors.reset_cursor_position(&line_stream.cursor.path);
              }
              info!("Setting reactor state to have no current character");
              self.state.update_reactor(|r| r.current_character = None);
            }
            Some(ReactorEvent::ExecEffect{effect, event_context}) => {
              self.exec_effect(effect, event_context).await;
            }
            Some(ReactorEvent::TestAudioFile(file_path)) => {
              let mixer = self.mixer.clone();
              spawn(async move {
                _ = mixer.play_file(&file_path).await;
              });
            }
          }
        }
        line_maybe = async {
          // When the line stream gets dropped, next() will receive None immediately
          debug!("awaiting next line...");
          line_stream_maybe.as_mut().unwrap().next().await // unwrap is infallible here due to is_some() check
        }, if line_stream_maybe.is_some() => {
          match line_maybe {
            Some((line, cursor_after)) => {
              debug!("LINE: {:?}", line);
              self.react_to_line(line, cursor_after).await; // TODO: can spawn be used here if self is an &Arc<Self>?
            },
            None => {
              debug!("Reactor encountered end of LogLineStream");
              if let Some(line_stream) = line_stream_maybe.take() {
                self.cursors.reset_cursor_position(&line_stream.cursor.path);
              }
              self.state.update_reactor(|r| r.current_character = None);
            }
          }
        }
      }
    }
    _ = self.log_events.stop();
    debug!("Event Loop finished");
  }

  async fn react_to_line(&self, line: Line, cursor_after: LogFileCursor) {
    self.state.with_reactor(|reactor_state| {
      self.state.select_triggers(|index| {
        let Some(character) = &reactor_state.current_character else {
          warn!("Cannot process line! No current character detected!");
          return;
        };
        let cursor_after = Arc::new(cursor_after);
        let active_triggers: Vec<&Trigger> =
          index.get_distinct_triggers_tagged_by_any_of(reactor_state.active_trigger_tags.iter());
        for trigger in active_triggers.into_iter() {
          if let Some(match_context) = trigger.filter.check(&line.content, &character.name) {
            let match_context = Arc::new(match_context);
            for effect in trigger.effects.iter() {
              debug!("TRIGGER EFFECT: {effect:?}");
              self.send(ReactorEvent::ExecEffect {
                effect: effect.clone(),
                event_context: self
                  .create_event_context(match_context.clone(), cursor_after.clone()),
              });
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

  pub fn create_event_context(
    &self,
    match_context: Arc<MatchContext>,
    cursor_after: Arc<LogFileCursor>,
  ) -> Arc<EventContext> {
    Arc::new(EventContext {
      reactor_tx: self.reactor_tx.clone(),
      timer_manager: self.timer_manager.clone(),
      overlay_manager: self.overlay_manager.clone(),
      mixer: self.mixer.clone(),
      t2s_tx: self.t2s_tx.clone(),
      match_context,
      cursor_after,
      timer_context: None,
      clipboard: self.clipboard.clone(),
      tx_log_file_events: self.log_events.sender(),
    })
  }

  async fn exec_effect(&self, effect_with_id: EffectWithID, event_context: Arc<EventContext>) {
    spawn(async move {
      if let Err(effect_error) = effect_with_id.effect.ready().fire(event_context).await {
        error!("Encountered error executing Effect: {effect_error:?}");
      }
    });
  }
}

impl EventContext {
  pub fn with_timer_context(&self, timer_context: TimerContext) -> Arc<Self> {
    Arc::new(Self {
      timer_context: Some(timer_context),
      ..self.clone()
    })
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
    _ = tx_.send(TTS::Quit).await;
  });

  tx
}
