use crate::{
  common::{
    duration::Duration,
    fatal_error,
    shutdown::quitter,
    timestamp::{ObservableTimestamp, Timestamp},
    UUID,
  },
  reactor::{EventContext, ReactorEvent},
  triggers::timers::{Timer, TimerStartPolicy},
};
use serde::Serialize;
use std::{
  collections::HashMap,
  sync::{atomic::Ordering, Arc},
};
use std::{iter::once, sync::atomic::AtomicBool};
use tauri::async_runtime::spawn;
use tokio::{select, sync::Notify};
use tokio::{
  sync::{broadcast, mpsc, oneshot},
  time::Instant,
};
use tracing::{debug, error, info};

const TIMER_COMMAND_CHANNEL_SIZE: usize = 50;
const TIMER_STATE_UPDATE_CHANNEL_SIZE: usize = 50;
const RESET_TIMER_CHANNEL_SIZE: usize = 5;

#[derive(Debug)]
pub struct TimerManager {
  tx_commands: mpsc::Sender<TimerCommand>,
}

pub enum TimerCommand {
  Begin(TimerLifetime),
  Terminate(UUID),
  SetHidden(UUID, bool),
  Restart(UUID),
  CreateSubscription(
    Arc<
      oneshot::Sender<(
        Vec<TimerLifetime>,
        Arc<broadcast::Receiver<TimerStateUpdate>>,
      )>,
    >,
  ),
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(tag = "variant", content = "value")]
#[ts(tag = "variant", content = "value")]
pub enum TimerStateUpdate {
  TimerAdded(TimerLifetime),
  TimerKilled(UUID),
  TimerHiddenUpdated(UUID, bool),
  TimerRestarted {
    id: UUID,
    start_time: Timestamp,
    end_time: Timestamp,
  },
}

/// This is used by a timer reaper task that kills the timer after its duration has
/// elapsed. The reaper does not need a specific "kill" event because it uses the
/// closure of its Sender (i.e. when it's dropped) as the signal that the reaper
/// should terminate.
struct ResetTimerEvent; // used by a timer reaper task

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
pub struct TimerLifetime {
  timer: Timer,
  id: UUID,
  name: String,
  is_hidden: bool,
  start_time: Timestamp,
  end_time: ObservableTimestamp,

  #[serde(skip)]
  #[ts(skip)]
  context: Arc<EventContext>,
  #[serde(skip)]
  #[ts(skip)]
  is_finished: Arc<AtomicBool>,
  #[serde(skip)]
  #[ts(skip)]
  notify_finished: Arc<Notify>,
}

impl TimerLifetime {
  fn terminate(&self) {
    self.is_finished.store(true, Ordering::Release);
    self.notify_finished.notify_waiters();
  }
}

#[derive(Debug, Clone)]
pub struct TimerContext {
  pub timer_id: UUID,
  pub end_time: ObservableTimestamp,
  sender: mpsc::Sender<TimerCommand>,
  is_finished: Arc<AtomicBool>,
  notify_finished: Arc<Notify>,
}

impl TimerContext {
  pub async fn finished(&self) {
    if self.is_finished.load(Ordering::Acquire) {
      return;
    }
    self.notify_finished.notified().await;
  }

  pub async fn send(
    &self,
    timer_command: TimerCommand,
  ) -> Result<(), mpsc::error::SendError<TimerCommand>> {
    self.sender.send(timer_command).await
  }

  pub async fn set_is_hidden(
    &self,
    is_hidden: bool,
  ) -> Result<(), mpsc::error::SendError<TimerCommand>> {
    self
      .send(TimerCommand::SetHidden(self.timer_id.clone(), is_hidden))
      .await
  }

  pub async fn restart(&self) -> Result<(), mpsc::error::SendError<TimerCommand>> {
    self
      .send(TimerCommand::Restart(self.timer_id.clone()))
      .await
  }

  pub async fn terminate(&self) -> Result<(), mpsc::error::SendError<TimerCommand>> {
    self
      .send(TimerCommand::Terminate(self.timer_id.clone()))
      .await
  }
}

type TimerLifetimesMap = HashMap<UUID, (TimerLifetime, mpsc::Sender<ResetTimerEvent>)>;

impl TimerManager {
  pub fn new() -> Self {
    let (tx_commands, rx_commands) = mpsc::channel::<TimerCommand>(TIMER_COMMAND_CHANNEL_SIZE);

    let (tx_state_updates, _rx_state_updates) =
      broadcast::channel::<TimerStateUpdate>(TIMER_STATE_UPDATE_CHANNEL_SIZE);

    spawn(event_loop(
      rx_commands,
      tx_commands.clone(),
      tx_state_updates.clone(),
    ));

    Self { tx_commands }
  }

  pub async fn send(
    &self,
    command: TimerCommand,
  ) -> Result<(), mpsc::error::SendError<TimerCommand>> {
    self.tx_commands.send(command).await
  }

  pub async fn start_timer(
    &self,
    timer: Timer,
    context: Arc<EventContext>,
  ) -> Result<UUID, mpsc::error::SendError<TimerCommand>> {
    let id = UUID::new();
    let name = timer.name_tmpl.render(&context.match_context);
    let start_time = Timestamp::now();
    let end_time = ObservableTimestamp::new(&start_time + &timer.duration);
    let context = context.to_owned();
    let is_finished = Arc::new(AtomicBool::new(false));
    let notify_finished = Arc::new(Notify::new());

    debug!("Starting Timer `{name}` with duration {:?}", timer.duration);

    let timer_lifetime = TimerLifetime {
      id: id.clone(),
      timer,
      name,
      start_time,
      end_time,
      context,
      is_finished,
      notify_finished,
      is_hidden: false,
    };

    self
      .tx_commands
      .send(TimerCommand::Begin(timer_lifetime))
      .await
      .map(|_| id)
  }

  /// This functions atomically obtains a snapshot of the `TimerLifetimes` and a
  /// `broadcast::Receiver` that was subscribed before any other changes could
  /// have been made to `TimerLifetimes`, guaranteeing the events can keep the shared
  /// state up-to-date (as long as the `broadcast::Receiver` doesn't become `Lagged`
  /// before it begins consuming events, in which case it must call this function
  /// again to get a fresh snapshot and a new up-to-date `broadcast::Receiver`).
  pub async fn subscribe(&self) -> (Vec<TimerLifetime>, broadcast::Receiver<TimerStateUpdate>) {
    let (setter, getter) = oneshot::channel::<(
      Vec<TimerLifetime>,
      Arc<broadcast::Receiver<TimerStateUpdate>>,
    )>();

    if let Err(_send_error) = self
      .tx_commands
      .send(TimerCommand::CreateSubscription(Arc::new(setter)))
      .await
    {
      fatal_error("Attempted to subscribe to TimerManager but its worker task has stopped");
    }

    let (timer_lifetimes, subscription) =
      getter.await.expect("TimerManager event loop appears dead");

    let subscription = Arc::into_inner(subscription).unwrap(); // unwrap is safe here

    (timer_lifetimes, subscription)
  }
}

async fn event_loop(
  mut rx_command: mpsc::Receiver<TimerCommand>,
  tx_command: mpsc::Sender<TimerCommand>,
  tx_state_update: broadcast::Sender<TimerStateUpdate>,
) {
  debug!("Starting TimerManager event loop");
  let mut timer_lifetimes: TimerLifetimesMap = HashMap::new();
  let mut quit = quitter();
  loop {
    select! {
      _ = &mut quit => {
        debug!("Timers event loop QUITTING");
        break;
      }
      command = rx_command.recv() => match command {
        None => break,
        Some(TimerCommand::CreateSubscription(setter)) => {
          let snapshot: Vec<TimerLifetime> = timer_lifetimes.values().map(|(t, _)| t.clone()).collect();
          let subscription = tx_state_update.subscribe();
          let setter = Arc::into_inner(setter).unwrap(); // unwrap is safe here
          let _ = setter.send((snapshot, Arc::new(subscription)));
        }
        Some(TimerCommand::Begin(timer_lifetime)) => {
          let TimerLifetime {
            id,
            timer,
            name,
            context,
            is_finished,
            notify_finished,
            end_time,
            ..
          } = &timer_lifetime;

          match &timer.start_policy {
            TimerStartPolicy::AlwaysStartNewTimer => { /* nothing to do here */ }
            TimerStartPolicy::DoNothingIfTimerRunning => {
              if is_timer_running_with_name(&name, &timer_lifetimes) {
                debug!("Timer[{id}] DoNothingIfTimerRunning policy [ name = `{name}` ]");
                return;
              }
            }
            TimerStartPolicy::StartAndReplacesAllTimersOfTrigger => {
              kill_timers_of_trigger(&timer.trigger_id, &mut timer_lifetimes, &tx_state_update);
            }
            TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerWithNameTemplateMatching(replaced_name_template) => {
              let replaced_name = replaced_name_template.render(&context.match_context);
              kill_timers_of_trigger_with_name(&timer.trigger_id, &replaced_name, &mut timer_lifetimes, &tx_state_update);
            }
          }

          let tx_reaper = spawn_timer_reaper(id.clone(), timer.duration.clone(), tx_command.clone());
          timer_lifetimes.insert(id.clone(), (timer_lifetime.clone(), tx_reaper));

          let _ = tx_state_update.send(TimerStateUpdate::TimerAdded(timer_lifetime.clone()));

          let context = context.with_timer_context(TimerContext {
            timer_id: id.clone(),
            sender: tx_command.clone(),
            end_time: end_time.clone(),
            is_finished: is_finished.clone(),
            notify_finished: notify_finished.clone(),
          });

          for effect in timer_lifetime.timer.effects.iter() {
            let _ = context.reactor_tx.send(ReactorEvent::ExecEffect {
              effect: effect.clone(),
              event_context: context.clone()
            }).await;
          }
        }
        Some(TimerCommand::Terminate(timer_lifetime_id)) => {
          kill_timers(once(&timer_lifetime_id), &mut timer_lifetimes, &tx_state_update);
        }
        Some(TimerCommand::Restart(timer_id)) => {
          if let Some((timer_lifetime, reaper_sender)) = timer_lifetimes.get(&timer_id) {
            let new_start_timestamp = Timestamp::now();
            let new_end_timestamp = &new_start_timestamp + &timer_lifetime.timer.duration;
            timer_lifetime.end_time.set(new_end_timestamp.clone());

            let _ = reaper_sender.send(ResetTimerEvent).await;
            let _ = tx_state_update.send(TimerStateUpdate::TimerRestarted {
              id: timer_id,
              start_time: new_start_timestamp,
              end_time: new_end_timestamp,
            });
          }
        }
        Some(TimerCommand::SetHidden(timer_id, is_hidden)) => {
          if let Some((timer_lifetime, _reaper_sender)) = timer_lifetimes.get_mut(&timer_id) {
            if timer_lifetime.is_hidden != is_hidden {
              timer_lifetime.is_hidden = is_hidden;
              let _ = tx_state_update.send(TimerStateUpdate::TimerHiddenUpdated(timer_lifetime.id.clone(), is_hidden));
            }
          }
        }
      }

    }
  }
  info!("Timer event loop stopped");
}

fn spawn_timer_reaper(
  timer_id: UUID,
  duration: Duration,
  tx_timer_event: mpsc::Sender<TimerCommand>,
) -> mpsc::Sender<ResetTimerEvent> {
  let (tx_reaper_event, mut rx_reaper_event) =
    mpsc::channel::<ResetTimerEvent>(RESET_TIMER_CHANNEL_SIZE);
  spawn(async move {
    debug!("Timer[{timer_id}] Reaper task spawned");

    let duration: tokio::time::Duration = duration.into();
    let mut end_instant: Instant = Instant::now() + duration;

    let mut quit = quitter();
    loop {
      let timer_id = timer_id.clone();
      select! {
        () = &mut quit => {
          debug!("Timer reaper QUITTING");
          break;
        }
        () = tokio::time::sleep_until(end_instant) => { // Instant implements Copy
          let _ = tx_timer_event.send(TimerCommand::Terminate(timer_id)).await;
          break;
        }
        event = rx_reaper_event.recv() => {
          match event {
            None => {
              // When the sender for a Timer reaper is dropped (i.e. when the timer is
              // removed from the event_loop HashMap), a None will be sent immediately,
              // indicating that the Timer has been killed.
              break;
            }
            Some(_reset_timer_event) => {
              end_instant = Instant::now() + duration;
            },
          }
        },
      }
    }
    debug!("Timer[{timer_id}] Reaper finished");
  });
  tx_reaper_event
}

fn is_timer_running_with_name(name: &str, timer_lifetimes: &TimerLifetimesMap) -> bool {
  timer_lifetimes.values().any(|(t, _)| t.name == name)
}

fn kill_timers<'a, I>(
  timer_ids: I,
  timer_lifetimes: &mut TimerLifetimesMap,
  tx_state_update: &broadcast::Sender<TimerStateUpdate>,
) where
  I: Iterator<Item = &'a UUID>,
{
  for timer_id in timer_ids {
    if let Some((timer_lifetime, _reaper_sender)) = timer_lifetimes.remove(timer_id) {
      timer_lifetime.terminate();
      let _ = tx_state_update.send(TimerStateUpdate::TimerKilled(timer_lifetime.id));
    } else {
      error!("Tried killing Timer[{timer_id}] but it wasn't found!");
    }
  }
}

fn kill_timers_of_trigger(
  trigger_id: &UUID,
  timer_lifetimes: &mut TimerLifetimesMap,
  tx_state_update: &broadcast::Sender<TimerStateUpdate>,
) {
  let trigger_ids = timer_lifetimes_for_trigger(trigger_id, timer_lifetimes);
  kill_timers(
    trigger_ids.iter().map(|tup| &tup.0),
    timer_lifetimes,
    tx_state_update,
  );
}

fn kill_timers_of_trigger_with_name(
  trigger_id: &UUID,
  timer_name: &str,
  timer_lifetimes: &mut TimerLifetimesMap,
  tx_state_update: &broadcast::Sender<TimerStateUpdate>,
) {
  let trigger_ids: Vec<UUID> = timer_lifetimes_for_trigger(trigger_id, timer_lifetimes)
    .into_iter()
    .filter_map(|(timer_id, each_timer_name)| {
      if timer_name == each_timer_name {
        Some(timer_id)
      } else {
        None
      }
    })
    .collect();
  kill_timers(trigger_ids.iter(), timer_lifetimes, tx_state_update);
}

/// Returns a vector of (<TIMER ID>, <TIMER NAME>) tuples
fn timer_lifetimes_for_trigger(
  trigger_id: &UUID,
  timer_lifetimes: &mut TimerLifetimesMap,
) -> Vec<(UUID, String)> {
  timer_lifetimes
    .values()
    .filter_map(|(life, _)| {
      if &life.timer.trigger_id == trigger_id {
        Some((life.id.clone(), life.name.clone()))
      } else {
        None
      }
    })
    .collect()
}
