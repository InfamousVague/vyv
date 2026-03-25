use crate::state::{AppState, AwakeMode};

pub fn activate(state: &AppState, mode: AwakeMode) -> Result<(), String> {
    let mut guard = state.awake_guard.lock().map_err(|e| e.to_string())?;
    let mut current_mode = state.awake_mode.lock().map_err(|e| e.to_string())?;

    // Drop existing guard first
    *guard = None;

    match mode {
        AwakeMode::Off => {
            *current_mode = AwakeMode::Off;
            return Ok(());
        }
        AwakeMode::DisplayAndSystem => {
            eprintln!("[Vyv] Activating: Display + System sleep prevention");
            let new_guard = keepawake::Builder::default()
                .display(true)
                .idle(true)
                .sleep(true)
                .reason("Vyv is keeping your computer awake")
                .app_name("Vyv")
                .app_reverse_domain("com.vyv.app")
                .create()
                .map_err(|e| {
                    eprintln!("[Vyv] keepawake error: {}", e);
                    format!("Failed to activate keepawake: {}", e)
                })?;
            eprintln!("[Vyv] keepawake guard created successfully");
            *guard = Some(new_guard);
        }
        AwakeMode::SystemOnly => {
            eprintln!("[Vyv] Activating: System-only sleep prevention");
            let new_guard = keepawake::Builder::default()
                .display(false)
                .idle(true)
                .sleep(true)
                .reason("Vyv is keeping your system awake (display may sleep)")
                .app_name("Vyv")
                .app_reverse_domain("com.vyv.app")
                .create()
                .map_err(|e| {
                    eprintln!("[Vyv] keepawake error: {}", e);
                    format!("Failed to activate keepawake: {}", e)
                })?;
            eprintln!("[Vyv] keepawake guard created successfully");
            *guard = Some(new_guard);
        }
    }

    *current_mode = mode;
    eprintln!("[Vyv] Mode set to: {}", mode);
    Ok(())
}

pub fn deactivate(state: &AppState) -> Result<(), String> {
    let mut guard = state.awake_guard.lock().map_err(|e| e.to_string())?;
    let mut mode = state.awake_mode.lock().map_err(|e| e.to_string())?;

    eprintln!("[Vyv] Deactivating keepawake");
    *guard = None;
    *mode = AwakeMode::Off;

    // Also cancel any running timer
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    if let Some(t) = timer.take() {
        t.cancel.cancel();
    }

    Ok(())
}
