#![allow(dead_code)]
use crate::state::{AppState, AwakeMode};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryAction {
    Deactivate,
    SwitchToSystemOnly,
}

pub fn start(
    threshold_percent: f32,
    action: BatteryAction,
    app: AppHandle,
    state: Arc<AppState>,
    cancel: CancellationToken,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30)) => {}
                _ = cancel.cancelled() => return,
            }

            if cancel.is_cancelled() {
                return;
            }

            let current_mode = *state.awake_mode.lock().unwrap();
            if current_mode == AwakeMode::Off {
                continue;
            }

            // Check battery level
            let battery_level = match get_battery_level() {
                Some(level) => level,
                None => continue, // No battery (desktop) — skip
            };

            if battery_level < threshold_percent {
                match action {
                    BatteryAction::Deactivate => {
                        let _ = crate::awake::deactivate(&state);
                        let _ = crate::tray::update_tray(&app, &state);
                    }
                    BatteryAction::SwitchToSystemOnly => {
                        if current_mode != AwakeMode::SystemOnly {
                            let _ = crate::awake::activate(&state, AwakeMode::SystemOnly);
                            let _ = crate::tray::update_tray(&app, &state);
                        }
                    }
                }
            }
        }
    })
}

fn get_battery_level() -> Option<f32> {
    let manager = battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let battery = batteries.next()?.ok()?;
    Some(battery.state_of_charge().get::<battery::units::ratio::percent>())
}
