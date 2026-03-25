use crate::state::{AppState, TimerPreset, TimerState};
use crate::tray;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub fn start_timer(
    app: AppHandle,
    state: &Arc<AppState>,
    preset: TimerPreset,
) -> Result<(), String> {
    let duration_secs = match preset.duration_secs() {
        Some(secs) => secs,
        None => {
            // Indefinite — no timer task needed, just store the state
            let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
            // Cancel existing timer if any
            if let Some(t) = timer.take() {
                t.cancel.cancel();
            }
            *timer = Some(TimerState {
                preset,
                started_at: Instant::now(),
                cancel: CancellationToken::new(),
            });
            return Ok(());
        }
    };

    let cancel = CancellationToken::new();

    {
        let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
        // Cancel existing timer if any
        if let Some(t) = timer.take() {
            t.cancel.cancel();
        }
        *timer = Some(TimerState {
            preset,
            started_at: Instant::now(),
            cancel: cancel.clone(),
        });
    }

    let state_clone = Arc::clone(state);
    tauri::async_runtime::spawn(async move {
        let duration = Duration::from_secs(duration_secs);
        tokio::select! {
            _ = tokio::time::sleep(duration) => {
                // Timer expired — deactivate
                let _ = crate::awake::deactivate(&state_clone);
                let _ = tray::update_tray(&app, &state_clone);
            }
            _ = cancel.cancelled() => {
                // Timer was cancelled (user deactivated or changed timer)
            }
        }
    });

    Ok(())
}

#[allow(dead_code)]
pub fn cancel_timer(state: &AppState) -> Result<(), String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    if let Some(t) = timer.take() {
        t.cancel.cancel();
    }
    Ok(())
}

pub fn remaining_secs(state: &AppState) -> Option<u64> {
    let timer = state.timer.lock().ok()?;
    let timer = timer.as_ref()?;
    let total = timer.preset.duration_secs()?;
    let elapsed = timer.started_at.elapsed().as_secs();
    if elapsed >= total {
        Some(0)
    } else {
        Some(total - elapsed)
    }
}

pub fn format_remaining(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
