use crate::{
  common::duration::Duration,
  common::shutdown::quitter,
  common::timestamp::Timestamp,
  common::{fatal_error, UUID},
  matchers::MatchContext,
  triggers::{Timer, TimerStartPolicy},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tauri::async_runtime::spawn;
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, error, info};

const TIMER_COMMAND_CHANNEL_SIZE: usize = 50;
const TIMER_STATE_UPDATE_CHANNEL_SIZE: usize = 50;
const RESET_TIMER_CHANNEL_SIZE: usize = 5;

pub struct TimerManager {
  tx_commands: mpsc::Sender<TimerCommand>,
}

#[derive(Debug, Clone)]
enum TimerCommand {
  StartLiveTimer(LiveTimer),
  LiveTimerElapsed(UUID),
  CreateSubscription(
    Arc<oneshot::Sender<(Vec<LiveTimer>, Arc<broadcast::Receiver<TimerStateUpdate>>)>>,
  ),
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub enum TimerStateUpdate {
  TimerAdded(LiveTimer),
  TimerKilled(UUID),
}

/// The reaper does not need a "kill" event because it uses the closure of its Sender
/// (i.e. when it's dropped) as the signal that the reaper should terminate.
struct ResetTimerEvent;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct LiveTimer {
  id: UUID,
  trigger_id: UUID,
  name: String,
  start_time: Timestamp,
  end_time: Timestamp,
  duration: Duration,
  repeats: bool,
  start_policy: TimerStartPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
struct LiveRepeatingTimer {
  name: String,
  start_time: Timestamp,
  interval_millis: usize,
}

type LiveTimersMap = HashMap<UUID, (LiveTimer, mpsc::Sender<ResetTimerEvent>)>;

async fn event_loop(
  mut rx_command: mpsc::Receiver<TimerCommand>,
  tx_command: mpsc::Sender<TimerCommand>,
  tx_state_update: broadcast::Sender<TimerStateUpdate>,
) {
  debug!("Starting TimerManager event loop");
  let mut live_timers: LiveTimersMap = HashMap::new();
  loop {
    match rx_command.recv().await {
      None => break,
      Some(TimerCommand::CreateSubscription(setter)) => {
        let snapshot: Vec<LiveTimer> = live_timers.values().map(|(live, _)| live.clone()).collect();
        let subscription = tx_state_update.subscribe();
        let setter = Arc::into_inner(setter).unwrap(); // unwrap is safe here
        let _ = setter.send((snapshot, Arc::new(subscription)));
      }
      Some(TimerCommand::StartLiveTimer(live_timer)) => {
        let LiveTimer {
          id,
          trigger_id,
          name,
          duration,
          start_policy,
          ..
        } = &live_timer;

        match start_policy {
          TimerStartPolicy::AlwaysStartNewTimer => { /* nothing to do here */ }
          TimerStartPolicy::DoNothingIfTimerRunning => {
            if is_timer_running_with_name(&name, &live_timers) {
              debug!("Timer[{id}] DoNothingIfTimerRunning policy [ name = `{name}` ]");
              return;
            }
          }
          TimerStartPolicy::StartAndReplacesAllTimersOfTrigger => {
            kill_timers_of_trigger(trigger_id, &mut live_timers);
          }
          TimerStartPolicy::StartAndReplacesAnyTimerOfTriggerHavingName(replaced_name) => {
            kill_timers_of_trigger_with_name(trigger_id, replaced_name, &mut live_timers);
          }
        }

        let tx_reaper = spawn_timer_reaper(id.clone(), duration.clone(), tx_command.clone());
        live_timers.insert(id.clone(), (live_timer.clone(), tx_reaper));
        let _ = tx_state_update.send(TimerStateUpdate::TimerAdded(live_timer));
      }
      Some(TimerCommand::LiveTimerElapsed(live_timer_id)) => {
        if let None = live_timers.remove(&live_timer_id) {
          error!("Timer[{live_timer_id}] COULD NOT BE REMOVED! DID NOT EXIST");
        }
        let _ = tx_state_update.send(TimerStateUpdate::TimerKilled(live_timer_id));
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
    let duration: std::time::Duration = duration.into();
    let mut quit = quitter();
    loop {
      let timer_id = timer_id.clone();
      let duration = duration.clone();
      select! {
        _ = &mut quit => {
          debug!("Timer reaper QUITTING");
          break;
        }
        _ = tokio::time::sleep(duration) => {
          let _ = tx_timer_event.send(TimerCommand::LiveTimerElapsed(timer_id)).await;
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
            Some(_reset_timer) => {
              // restarting the loop causes the sleep() future to be re-created anew
              continue;
            },
          }
        },
      }
    }
    debug!("Timer[{timer_id}] Reaper finished");
  });
  tx_reaper_event
}

impl TimerManager {
  pub fn new() -> Self {
    let (tx_commands, rx_commands) = mpsc::channel::<TimerCommand>(TIMER_COMMAND_CHANNEL_SIZE);

    let (tx_state_updates, _) =
      broadcast::channel::<TimerStateUpdate>(TIMER_STATE_UPDATE_CHANNEL_SIZE);

    spawn(event_loop(
      rx_commands,
      tx_commands.clone(),
      tx_state_updates.clone(),
    ));

    Self { tx_commands }
  }

  /// This functions atomically obtains a snapshot of the LiveTimers and a
  /// broadcast::Receiver that was subscribed before any other changes could
  /// have been made to LiveTimers, guaranteeing the events can keep the shared
  /// state up-to-date (as long as the broadcast::Receiver doesn't become Lagged
  /// before it begins consuming events, in which case it must call this function
  /// again to get a fresh snapshot and new up-to-date broadcast::Receiver).
  pub async fn subscribe(&self) -> (Vec<LiveTimer>, broadcast::Receiver<TimerStateUpdate>) {
    let (setter, getter) =
      oneshot::channel::<(Vec<LiveTimer>, Arc<broadcast::Receiver<TimerStateUpdate>>)>();

    if let Err(_send_error) = self
      .tx_commands
      .send(TimerCommand::CreateSubscription(Arc::new(setter)))
      .await
    {
      fatal_error("Attempted to subscribe to TimerManager but its worker task has stopped");
    }

    let (live_timers, subscription) = getter.await.expect("TimerManager event loop appears dead");
    let subscription = Arc::into_inner(subscription).unwrap(); // unwrap is safe here
    (live_timers, subscription)
  }

  pub async fn start_timer(
    &self,
    Timer {
      trigger_id,
      name,
      duration,
      start_policy,
      repeats,
      // tags, updates,
      ..
    }: &Timer,
    context: &MatchContext,
  ) {
    let name = name.render(context);
    let start_time = Timestamp::now();
    let end_time = &start_time + duration;

    let live_timer = LiveTimer {
      id: UUID::new(),
      trigger_id: trigger_id.clone(),
      name,
      start_time,
      end_time,
      repeats: *repeats,
      duration: duration.to_owned(),
      start_policy: start_policy.clone(),
    };

    if let Err(_send_error) = self
      .tx_commands
      .send(TimerCommand::StartLiveTimer(live_timer))
      .await
    {
      error!("Tried to send StartLiveTimer, but the channel was closed");
    }
  }
}

fn is_timer_running_with_name(name: &str, live_timers: &LiveTimersMap) -> bool {
  live_timers.values().any(|(live, _)| live.name == name)
}

fn kill_timers<'a, I>(timer_ids: I, live_timers: &mut LiveTimersMap)
where
  I: Iterator<Item = &'a UUID>,
{
  // TODO: SEND DOWNSTREAM CHANGE EVENTS
  for timer_id in timer_ids {
    if let None = live_timers.remove(timer_id) {
      error!("Tried killing Timer[{timer_id}] but it wasn't found!");
    }
  }
}

fn kill_timers_of_trigger(trigger_id: &UUID, live_timers: &mut LiveTimersMap) {
  let trigger_ids = live_timers_for_trigger(trigger_id, live_timers);
  kill_timers(trigger_ids.iter().map(|tup| &tup.0), live_timers);
}

fn kill_timers_of_trigger_with_name(
  trigger_id: &UUID,
  timer_name: &str,
  live_timers: &mut LiveTimersMap,
) {
  let trigger_ids: Vec<UUID> = live_timers_for_trigger(trigger_id, live_timers)
    .into_iter()
    .filter_map(|(timer_id, each_timer_name)| {
      if timer_name == each_timer_name {
        Some(timer_id)
      } else {
        None
      }
    })
    .collect();
  kill_timers(trigger_ids.iter(), live_timers);
}

/// Returns a vector of (<TIMER ID>, <TIMER NAME>) tuples
fn live_timers_for_trigger(
  trigger_id: &UUID,
  live_timers: &mut LiveTimersMap,
) -> Vec<(UUID, String)> {
  live_timers
    .values()
    .filter_map(|(live, _)| {
      if &live.trigger_id == trigger_id {
        Some((live.id.clone(), live.name.clone()))
      } else {
        None
      }
    })
    .collect()
}
