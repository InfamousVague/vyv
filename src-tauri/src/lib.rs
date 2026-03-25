mod awake;
mod battery_monitor;
#[cfg(target_os = "macos")]
mod clamshell;
mod process_monitor;
mod profiles;
mod simulation;
mod state;
mod stats;
mod timer;
mod tray;

use state::AppState;
use stats::StatsTracker;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState::new());
    let stats_tracker = Arc::new(StatsTracker::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Phase 4: handle CLI args forwarded from second instance
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .build(),
        )
        .manage(app_state.clone())
        .manage(stats_tracker.clone())
        .setup(move |app| {
            // Build the system tray
            tray::build_tray(app, &app_state, &stats_tracker)?;

            // Register panic hotkey (Ctrl+Shift+Escape)
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            let state_for_hotkey = app_state.clone();
            let app_handle = app.handle().clone();
            app.global_shortcut().on_shortcut("ctrl+shift+escape", move |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    // Panic button: stop simulation and deactivate
                    let mut sim = state_for_hotkey.simulation.lock().unwrap();
                    if let Some(s) = sim.take() {
                        s.cancel.cancel();
                    }
                    let _ = crate::awake::deactivate(&state_for_hotkey);
                    let _ = tray::update_tray(&app_handle, &state_for_hotkey);
                }
            })?;

            // On macOS, hide the dock icon (tray-only app)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Vyv");
}
