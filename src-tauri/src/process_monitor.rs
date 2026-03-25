#![allow(dead_code)]
use crate::state::{AppState, AwakeMode, TimerPreset};
use crate::timer;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub struct ProcessMonitorHandle {
    pub cancel: CancellationToken,
}

pub fn start(
    watched: Vec<String>,
    app: AppHandle,
    state: Arc<AppState>,
    cancel: CancellationToken,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        let mut auto_activated = false;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                _ = cancel.cancelled() => return,
            }

            if cancel.is_cancelled() {
                return;
            }

            // Refresh process list
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

            let any_match = sys.processes().values().any(|proc| {
                let name = proc.name().to_string_lossy().to_lowercase();
                watched.iter().any(|w| name.contains(&w.to_lowercase()))
            });

            let current_mode = *state.awake_mode.lock().unwrap();

            if any_match && current_mode == AwakeMode::Off {
                // Auto-activate
                let _ = crate::awake::activate(&state, AwakeMode::DisplayAndSystem);
                let _ = timer::start_timer(app.clone(), &state, TimerPreset::Indefinite);
                let _ = crate::tray::update_tray(&app, &state);
                auto_activated = true;
            } else if !any_match && auto_activated && current_mode != AwakeMode::Off {
                // Auto-deactivate (only if we auto-activated, not user-activated)
                let _ = crate::awake::deactivate(&state);
                let _ = crate::tray::update_tray(&app, &state);
                auto_activated = false;
            }
        }
    })
}
