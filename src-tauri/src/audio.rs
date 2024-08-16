use crate::common::{fatal_error, shutdown::quitter};
use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use cpal::traits::{DeviceTrait as _, HostTrait as _};
use std::path::PathBuf;
use std::thread;
use tauri::async_runtime::spawn;
use tokio::sync::mpsc;
use tracing::{debug, error};

const AUDIO_MIXER_CHANNEL_SIZE: usize = 10;

pub struct AudioMixer {
  #[allow(unused)]
  join_handle: thread::JoinHandle<()>,
  sender: mpsc::Sender<AudioMixerEvent>,
}

enum AudioMixerEvent {
  PlayFile(PathBuf),
  #[allow(unused)]
  Reset,
  Terminate,
}

#[derive(thiserror::Error, Debug)]
#[error("Sound file does not exist to play: {0}")]
pub struct PlayAudioFileError(PathBuf);

impl AudioMixer {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel::<AudioMixerEvent>(AUDIO_MIXER_CHANNEL_SIZE);

    let join_handle = thread::Builder::new()
      .name("LogQuest AudioMixer".into())
      .spawn(move || mixer_loop(rx))
      .expect("Cannot create AudioMixer thread!"); // panic-worthy

    let tx_ = tx.clone();
    spawn(async move {
      quitter().await;
      let _ = tx_.send(AudioMixerEvent::Terminate).await;
    });

    Self {
      join_handle,
      sender: tx,
    }
  }

  pub fn play_file(&self, file_path: &str) -> Result<(), PlayAudioFileError> {
    let file_path: PathBuf = file_path.into();
    if !file_path.is_file() {
      return Err(PlayAudioFileError(file_path));
    }

    let sender = self.sender.clone();
    spawn(async move {
      if let Err(_send_error) = sender.send(AudioMixerEvent::PlayFile(file_path)).await {
        error!("Could not send PlayFile message to the AudioMixer!");
      }
    });

    Ok(())
  }

  //// Leaving these commented out until they're used
  //
  // pub async fn interrupt(&self) {
  //   let _ = self.sender.send(AudioMixerEvent::Reset).await;
  // }
  //
  // pub fn terminate_blocking(self) {
  //   let _ = self.sender.blocking_send(AudioMixerEvent::Terminate);
  //   let _ = self.join_handle.join();
  // }
  //
}

fn mixer_loop(mut rx: mpsc::Receiver<AudioMixerEvent>) {
  // The CpalBackend value needs to be kept around for the audio engine to work.
  let player: (Manager, CpalBackend) =
    awedio::start().expect("Could not create the audio backend and manager!"); // panic-worthy

  let mut manager = player.0;

  debug!("Starting AudioMixer event loop");
  loop {
    match rx.blocking_recv() {
      Some(AudioMixerEvent::Reset) => {
        debug!("AudioMixer reset");
        manager.clear()
      }
      Some(AudioMixerEvent::Terminate) | None => {
        debug!("AudioMixer terminated");
        return;
      }
      Some(AudioMixerEvent::PlayFile(next_file)) => {
        if let Ok(file) = awedio::sounds::open_file(&next_file) {
          debug!("Playing audio file: {}", next_file.display());
          manager.play(file);
        } else {
          error!("Could not open audio file: {}", next_file.display());
        }
      }
    }
  }
}

pub fn get_device_names() -> Vec<String> {
  let host = cpal::platform::default_host();
  let Ok(devices) = host.devices() else {
    fatal_error("Could not get a list of the audio devices!");
  };
  devices
    .map(|device| device.name().unwrap_or_else(|_| "[UNKNOWN]".into()))
    .collect()
}

pub fn print_audio_devices() -> ! {
  let devices = get_device_names();
  if devices.is_empty() {
    fatal_error("No audio devices found!");
  }
  println!("\nAudio devices detected:\n");
  for device_name in devices.iter() {
    println!(" - {device_name}");
  }
  println!("");
  std::process::exit(0);
}
