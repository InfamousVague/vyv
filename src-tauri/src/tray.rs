use crate::profiles::SimProfile;
use crate::state::{AppState, AwakeMode, DriveAliveState, SimulationState, TimerPreset};
use crate::stats::StatsTracker;
use crate::timer;
use std::sync::Arc;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::image::Image;
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

const TRAY_ID: &str = "vyv-tray";

fn load_tray_icon(active: bool) -> Image<'static> {
    let bytes: &[u8] = if active {
        include_bytes!("../icons/tray-active.png")
    } else {
        include_bytes!("../icons/tray-inactive.png")
    };
    Image::from_bytes(bytes).expect("Failed to load tray icon")
}

pub fn build_tray(
    app: &tauri::App,
    state: &Arc<AppState>,
    stats: &Arc<StatsTracker>,
) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(&app.handle(), state, stats)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(load_tray_icon(false))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Vyv — Off")
        .on_menu_event({
            let state = Arc::clone(state);
            let stats = Arc::clone(stats);
            move |app, event| {
                handle_menu_event(app, &state, &stats, event.id().as_ref());
            }
        })
        .build(app)?;

    Ok(())
}

fn build_menu(
    app: &AppHandle,
    state: &Arc<AppState>,
    stats: &Arc<StatsTracker>,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mode = *state.awake_mode.lock().unwrap();
    let guard_held = state.awake_guard.lock().unwrap().is_some();
    let is_active = guard_held; // Session is active only if keepawake guard is held
    let remaining = timer::remaining_secs(state);
    let sim_active = state.simulation.lock().unwrap().is_some();
    let sim_profile_name = state
        .simulation
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.profile_name.clone());

    // ── Status line ─────────────────────────────────────────
    let status_text = if is_active {
        let mode_str = match mode {
            AwakeMode::DisplayAndSystem => "Display + System",
            AwakeMode::SystemOnly => "System Only",
            AwakeMode::Off => unreachable!(),
        };
        let time = remaining
            .map(timer::format_remaining)
            .unwrap_or_else(|| "Indefinite".into());
        format!("Vyv: {} — {}", mode_str, time)
    } else {
        "Vyv: Off".to_string()
    };
    let status = MenuItemBuilder::with_id("status", &status_text)
        .enabled(false)
        .build(app)?;

    // ── Start/End Session ───────────────────────────────────
    let mut menu_builder = MenuBuilder::new(app).item(&status).separator();

    if is_active {
        menu_builder = menu_builder
            .item(&MenuItemBuilder::with_id("end_session", "End Session").build(app)?);
    } else {
        menu_builder = menu_builder
            .item(
                &MenuItemBuilder::with_id("start_header", "Start New Session:")
                    .enabled(false)
                    .build(app)?,
            )
            .item(&MenuItemBuilder::with_id("timer_indefinite", "  Indefinitely").build(app)?);

        let minutes_sub = SubmenuBuilder::with_id(app, "minutes_sub", "  Minutes")
            .item(&MenuItemBuilder::with_id("timer_min5", "5 Minutes").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_min10", "10 Minutes").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_min15", "15 Minutes").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_min30", "30 Minutes").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_min45", "45 Minutes").build(app)?)
            .build()?;
        menu_builder = menu_builder.item(&minutes_sub);

        let hours_sub = SubmenuBuilder::with_id(app, "hours_sub", "  Hours")
            .item(&MenuItemBuilder::with_id("timer_hour1", "1 Hour").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_hour2", "2 Hours").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_hour4", "4 Hours").build(app)?)
            .item(&MenuItemBuilder::with_id("timer_hour8", "8 Hours").build(app)?)
            .build()?;
        menu_builder = menu_builder.item(&hours_sub);
    }

    // ── Settings ── (built below, but we need sim state for it)
    let _ = sim_profile_name; // suppress warning

    // ── Settings ────────────────────────────────────────────
    let keeping_display = mode == AwakeMode::DisplayAndSystem;

    // ── Drive Alive submenu ────────────────────────────────
    let active_drives: Vec<String> = state
        .drive_alive
        .lock()
        .unwrap()
        .as_ref()
        .map(|da| da.drives.clone())
        .unwrap_or_default();

    let available_drives = crate::drive_alive::list_drives();
    let mut drive_sub = SubmenuBuilder::with_id(app, "drive_alive_sub", "Drive Alive");

    if available_drives.is_empty() {
        drive_sub = drive_sub.item(
            &MenuItemBuilder::with_id("da_none", "No external drives found")
                .enabled(false)
                .build(app)?,
        );
    } else {
        for drive in &available_drives {
            let name = drive
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let id = format!("da_drive_{}", name);
            let is_active = active_drives.contains(&name);
            drive_sub = drive_sub.item(
                &CheckMenuItemBuilder::with_id(id, &name)
                    .checked(is_active)
                    .build(app)?,
            );
        }
    }

    let drive_alive_menu = drive_sub.build()?;

    let settings = SubmenuBuilder::with_id(app, "settings_submenu", "Settings")
        .item(
            &CheckMenuItemBuilder::with_id("qs_keep_display", "Keep display awake")
                .checked(keeping_display)
                .build(app)?,
        )
        .item(
            &CheckMenuItemBuilder::with_id(
                "qs_prevent_clamshell",
                "Prevent sleep when lid closed",
            )
            .checked(crate::clamshell::is_enabled())
            .build(app)?,
        )
        .item(
            &CheckMenuItemBuilder::with_id("qs_jiggle", "Jiggle mouse to prevent idle")
                .checked(sim_active)
                .build(app)?,
        )
        .item(&drive_alive_menu)
        .build()?;

    // ── Stats ───────────────────────────────────────────────
    let total_time = stats.format_total();
    let dosage = stats.get_dosage_level();

    menu_builder = menu_builder
        .separator()
        .item(&settings)
        .separator()
        .item(
            &MenuItemBuilder::with_id("stats", format!("Total: {} ({})", total_time, dosage))
                .enabled(false)
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("quit", "Quit Vyv").build(app)?);

    let menu = menu_builder.build()?;
    Ok(menu)
}

pub fn update_tray(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    // We need stats too — get it from managed state
    let stats = app
        .try_state::<Arc<StatsTracker>>()
        .ok_or("Stats not available")?;

    let tray = app.tray_by_id(TRAY_ID).ok_or("Tray icon not found")?;

    let menu = build_menu(app, state, &stats).map_err(|e| e.to_string())?;
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;

    let mode = *state.awake_mode.lock().unwrap();
    let tooltip = match mode {
        AwakeMode::Off => "Vyv — Off".to_string(),
        _ => {
            let remaining = timer::remaining_secs(state);
            match remaining {
                Some(secs) => format!("Vyv — {} ({})", mode, timer::format_remaining(secs)),
                None => format!("Vyv — {} (Indefinite)", mode),
            }
        }
    };
    tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;

    // Swap tray icon based on state
    let is_active = mode != AwakeMode::Off;
    tray.set_icon(Some(load_tray_icon(is_active))).map_err(|e| e.to_string())?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, state: &Arc<AppState>, stats: &Arc<StatsTracker>, id: &str) {
    match id {
        "end_session" => {
            stop_simulation(state);
            stop_drive_alive(state);
            stats.end_session();
            let _ = crate::clamshell::disable(); // Re-enable normal sleep
            let _ = crate::awake::deactivate(state);
            let _ = update_tray(app, state);
        }

        "quit" => {
            stop_simulation(state);
            stop_drive_alive(state);
            stats.end_session();
            let _ = crate::clamshell::disable(); // Re-enable normal sleep
            let _ = crate::awake::deactivate(state);
            app.exit(0);
        }

        // Keep display awake toggle — just flips the preference
        "qs_keep_display" => {
            let current = *state.awake_mode.lock().unwrap();

            let new_mode = match current {
                AwakeMode::DisplayAndSystem => AwakeMode::SystemOnly,
                AwakeMode::SystemOnly | AwakeMode::Off => AwakeMode::DisplayAndSystem,
            };

            // Store the preferred mode — only apply if session is active
            if current != AwakeMode::Off {
                let preset = current_or_default_preset(state);
                let _ = crate::awake::activate(state, new_mode);
                let state_arc = Arc::clone(state);
                let _ = timer::start_timer(app.clone(), &state_arc, preset);
            } else {
                // Just update the preference in state without starting a session
                *state.awake_mode.lock().unwrap() = new_mode;
            }
            let _ = update_tray(app, state);
        }

        // Prevent clamshell sleep toggle
        "qs_prevent_clamshell" => {
            if crate::clamshell::is_enabled() {
                let _ = crate::clamshell::disable();
            } else {
                let _ = crate::clamshell::enable();
            }
            let _ = update_tray(app, state);
        }

        // Mouse jiggle toggle
        "qs_jiggle" => {
            let sim = state.simulation.lock().unwrap();
            let is_running = sim.is_some();
            drop(sim);

            if is_running {
                stop_simulation(state);
            } else {
                start_simulation(state, &SimProfile::Slack); // Generic jiggle
            }
            let _ = update_tray(app, state);
        }

        // Drive Alive toggle
        id if id.starts_with("da_drive_") => {
            let drive_name = id.strip_prefix("da_drive_").unwrap().to_string();
            toggle_drive_alive(state, &drive_name);
            let _ = update_tray(app, state);
        }

        // Timer selections — start a session
        id if id.starts_with("timer_") => {
            let preset_key = id.strip_prefix("timer_").unwrap();
            let preset = match preset_key {
                "min5" => TimerPreset::Min5,
                "min10" => TimerPreset::Min10,
                "min15" => TimerPreset::Min15,
                "min30" => TimerPreset::Min30,
                "min45" => TimerPreset::Min45,
                "hour1" => TimerPreset::Hour1,
                "hour2" => TimerPreset::Hour2,
                "hour4" => TimerPreset::Hour4,
                "hour8" => TimerPreset::Hour8,
                "indefinite" => TimerPreset::Indefinite,
                _ => return,
            };

            stats.start_session();
            // Use the preferred mode (user may have pre-selected in settings)
            let preferred = *state.awake_mode.lock().unwrap();
            let mode = if preferred == AwakeMode::Off {
                AwakeMode::DisplayAndSystem
            } else {
                preferred
            };
            let _ = crate::awake::activate(state, mode);
            let state_arc = Arc::clone(state);
            let _ = timer::start_timer(app.clone(), &state_arc, preset);
            let _ = update_tray(app, state);
        }

        _ => {}
    }
}

fn start_simulation(state: &Arc<AppState>, profile: &SimProfile) {
    // Stop existing simulation first
    stop_simulation(state);

    let cancel = CancellationToken::new();
    crate::simulation::start(profile.clone(), cancel.clone());

    let mut sim = state.simulation.lock().unwrap();
    *sim = Some(SimulationState {
        cancel,
        profile_name: profile.label().to_string(),
    });
}

fn stop_simulation(state: &Arc<AppState>) {
    let mut sim = state.simulation.lock().unwrap();
    if let Some(s) = sim.take() {
        s.cancel.cancel();
    }
}

fn toggle_drive_alive(state: &Arc<AppState>, drive_name: &str) {
    let mut da = state.drive_alive.lock().unwrap();
    let mut drives: Vec<String> = da.as_ref().map(|d| d.drives.clone()).unwrap_or_default();

    if drives.contains(&drive_name.to_string()) {
        drives.retain(|d| d != drive_name);
    } else {
        drives.push(drive_name.to_string());
    }

    // Cancel existing task
    if let Some(old) = da.take() {
        old.cancel.cancel();
    }

    if drives.is_empty() {
        // No drives selected, leave state as None
        return;
    }

    // Start new task with updated drive list
    let cancel = CancellationToken::new();
    let paths: Vec<std::path::PathBuf> = drives
        .iter()
        .map(|name| std::path::PathBuf::from(format!("/Volumes/{}", name)))
        .collect();
    crate::drive_alive::start(paths, cancel.clone());

    *da = Some(DriveAliveState {
        cancel,
        drives,
    });
}

fn stop_drive_alive(state: &Arc<AppState>) {
    let mut da = state.drive_alive.lock().unwrap();
    if let Some(old) = da.take() {
        old.cancel.cancel();
    }
}

fn current_or_default_preset(state: &Arc<AppState>) -> TimerPreset {
    state
        .timer
        .lock()
        .ok()
        .and_then(|t| t.as_ref().map(|ts| ts.preset))
        .unwrap_or(TimerPreset::Indefinite)
}
