use crate::{
  commands,
  common::shutdown::shutdown,
  common::{fatal_error, ternary},
  reactor,
  state::{
    overlay::{OverlayManager, OverlayMode, OVERLAY_EDITABLE_CHANGED_EVENT_NAME},
    state_handle::StateHandle,
    timer_manager::TimerManager,
  },
};
use std::sync::Arc;
use tauri::async_runtime::spawn;
use tauri::App;
use tauri::{AppHandle, GlobalShortcutManager, Manager, Window, WindowEvent};
use tracing::{debug, error, info};

pub type OverlayManagerState = Arc<OverlayManager>;

#[cfg_attr(not(debug_assertions), allow(unused))]
pub const MAIN_WINDOW_LABEL: &str = "main";
pub const OVERLAY_WINDOW_LABEL: &str = "overlay";

pub const PROGRESS_UPDATE_EVENT_NAME: &str = "progress-update";
pub const PROGRESS_UPDATE_FINISHED_EVENT_NAME: &str = "progress-update-finished";

const TOGGLE_OVERLAY_ACCELERATOR: &str = "CommandOrControl+Alt+Shift+L";

pub fn launch(state: StateHandle) {
  let result = tauri::Builder::default()
    .manage(state.clone())
    .setup(move |app: &mut App| {
      let app_handle = app.app_handle();
      let timer_manager = create_timer_manager();
      let overlay_manager = create_overlay_manager(&app_handle, &timer_manager);
      app.manage(overlay_manager.clone() as OverlayManagerState);
      reactor(&app_handle, &state, timer_manager, overlay_manager);
      setup(&app_handle);
      Ok(())
    })
    .invoke_handler(commands::handler())
    .on_window_event(|window_event| {
      if let WindowEvent::CloseRequested { .. } = window_event.event() {
        let window = window_event.window();
        debug!("Window `{}` was closed. Shutting down...", window.label());
        shutdown();
        window.app_handle().exit(0);
      }
    })
    .run(tauri::generate_context!());

  if let Err(e) = result {
    fatal_error(e);
  }
}

fn create_timer_manager() -> Arc<TimerManager> {
  Arc::new(TimerManager::new())
}

fn create_overlay_manager(
  app: &AppHandle,
  timer_manager: &Arc<TimerManager>,
) -> Arc<OverlayManager> {
  Arc::new(OverlayManager::new(app.clone(), timer_manager.clone()))
}

fn reactor(
  app: &AppHandle,
  state: &StateHandle,
  timer_manager: Arc<TimerManager>,
  overlay_manager: Arc<OverlayManager>,
) {
  let reactor_started_future =
    reactor::start_when_config_is_ready(state, timer_manager, overlay_manager);
  let app = app.clone();
  spawn(async move {
    match reactor_started_future.await {
      Ok(Ok(tx_reactor)) => {
        app.manage(tx_reactor);
        debug!("Reactor started from UI");
      }
      Err(_recv_error) => {
        fatal_error("Reactor start_when_config_is_ready future channel closed?");
      }
      Ok(Err(reactor_start_error)) => {
        fatal_error(reactor_start_error.to_string());
      }
    }
  });
}

fn setup(app: &AppHandle) {
  register_global_shortcut_manager(app);
  setup_overlay(app);
  #[cfg(debug_assertions)]
  {
    if let Some(main) = app.get_window(MAIN_WINDOW_LABEL) {
      // inject the script to integrate with react-devtools
      if let Err(e) = main.eval(
        r#"(() => {
          const injection = document.createElement("script");
          injection.src = "http://localhost:8097";
          document.head.appendChild(injection);
        })();"#,
      ) {
        error!("Could not inject react-devtools integration script! Error: {e:?}");
      }
    }
  }
}

fn setup_overlay(app: &AppHandle) {
  let overlay_mode = state(app).select_overlay(|o| o.overlay_mode.clone());
  match overlay_mode {
    OverlayMode::Default => {
      create_default_overlay_window(app);
    }
    OverlayMode::Windowed => {
      create_windowed_overlay_window(app);
    }
    OverlayMode::None => {}
  }
}

fn create_default_overlay_window(app: &AppHandle) -> tauri::Window {
  let window_uri = tauri::WindowUrl::App("overlay.html".into());
  let overlay_window = tauri::WindowBuilder::new(app, OVERLAY_WINDOW_LABEL, window_uri)
    .title("LogQuest Overlay")
    .transparent(true)
    .fullscreen(true)
    .always_on_top(true)
    .focused(false)
    .skip_taskbar(true)
    .maximizable(false)
    .minimizable(false)
    .resizable(false)
    .accept_first_mouse(true)
    .build()
    .expect("Could not create overlay window!");

  // decorations must be disabled AFTER creating the window to avoid the M.O.A.B (Mother Of All Bugs) on Windows
  // that stole weeks of my life. If you create the window with both fullscreen(true) and decorations(true), then
  // a semi-transparent titlebar appears that cannot be removed, even when changing the low-level window styles at
  // runtime. I am pretty sure this is a bug with Windows itself.
  _ = overlay_window.set_decorations(false);

  let is_editable = state(app).select_overlay(|overlay| overlay.overlay_editable);
  set_overlay_editable(app, is_editable);

  overlay_window
}

fn create_windowed_overlay_window(app: &AppHandle) -> tauri::Window {
  let window_uri = tauri::WindowUrl::App("overlay.html".into());
  let overlay_window = tauri::WindowBuilder::new(app, OVERLAY_WINDOW_LABEL, window_uri)
    .title("LogQuest Overlay")
    .focused(false)
    .build()
    .expect("Could not create overlay window!");

  #[cfg(debug_assertions)]
  if state(app).select_overlay(|o| o.auto_open_dev_tools) {
    overlay_window.open_devtools();
  }

  overlay_window
}

// fn get_main_window(app: &AppHandle) -> Window {
//   // TODO: If I ever allow the main window be fully closed, this would panic...
//   app
//     .get_window("main")
//     .expect("Expected main window to exist!")
// }

fn get_overlay_window(app: &AppHandle) -> Option<Window> {
  let overlay_mode = state(app).select_overlay(|o| o.overlay_mode.clone());
  if let OverlayMode::None = overlay_mode {
    return None;
  }
  app.get_window(OVERLAY_WINDOW_LABEL)
}

fn register_global_shortcut_manager(app: &AppHandle) {
  let app = app.clone();
  app
    .global_shortcut_manager()
    .register(TOGGLE_OVERLAY_ACCELERATOR, move || {
      toggle_overlay_editable(&app)
    })
    .expect("Failed registering a global shortcut!");
}

fn toggle_overlay_editable(app: &AppHandle) {
  let inverse = state(app).select_overlay(|o| !o.overlay_editable);
  set_overlay_editable(app, inverse);
}

fn set_overlay_editable(app: &AppHandle, new_value: bool) {
  let state = state(app);
  state.update_overlay(|overlay| overlay.overlay_editable = new_value);
  let Some(overlay_window) = get_overlay_window(app) else {
    return;
  };
  overlay_window
    .set_ignore_cursor_events(!new_value)
    .expect("Failed to set_ignore_cursor_events");
  if new_value {
    if let Err(e) = overlay_window.set_focus() {
      error!("Failed to focus overlay window: {e:?}");
    }
  }
  _ = app.emit_all(OVERLAY_EDITABLE_CHANGED_EVENT_NAME, new_value);
  info!(
    "Overlay editing {}",
    ternary(new_value, "ENABLED", "DISABLED")
  )
}

fn state(app: &AppHandle) -> tauri::State<StateHandle> {
  app.state::<StateHandle>()
}
