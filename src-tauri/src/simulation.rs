#![allow(dead_code)]
use crate::profiles::SimProfile;
use enigo::{Enigo, Keyboard, Mouse, Settings as EnigoSettings};
use enigo::{Direction, Key};
use rand::Rng;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct SimulationHandle {
    pub profile: SimProfile,
    pub cancel: CancellationToken,
}

pub fn start(profile: SimProfile, cancel: CancellationToken) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        run_simulation_loop(&profile, &cancel).await;
    })
}

async fn run_simulation_loop(profile: &SimProfile, cancel: &CancellationToken) {
    let (min_ms, max_ms) = profile.interval_range();

    loop {
        // Wait a random interval
        let delay = {
            let mut rng = rand::thread_rng();
            rng.gen_range(min_ms..=max_ms)
        };

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(delay)) => {}
            _ = cancel.cancelled() => {
                return;
            }
        }

        if cancel.is_cancelled() {
            return;
        }

        // Perform the simulation action on a blocking thread
        // (enigo is not async and needs to run on a real thread)
        let do_mouse = profile.mouse_movement();
        let do_keyboard = profile.keyboard_input();

        let _ = tokio::task::spawn_blocking(move || {
            simulate_action(do_mouse, do_keyboard);
        })
        .await;
    }
}

fn simulate_action(do_mouse: bool, do_keyboard: bool) {
    let mut enigo = match Enigo::new(&EnigoSettings::default()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create enigo instance: {}", e);
            return;
        }
    };

    let mut rng = rand::thread_rng();

    if do_mouse {
        // Small random mouse movement — move a few pixels then back
        let dx: i32 = rng.gen_range(-3..=3);
        let dy: i32 = rng.gen_range(-3..=3);

        let _ = enigo.move_mouse(dx, dy, enigo::Coordinate::Rel);

        // Brief pause then move back
        std::thread::sleep(Duration::from_millis(rng.gen_range(50..150)));
        let _ = enigo.move_mouse(-dx, -dy, enigo::Coordinate::Rel);
    }

    if do_keyboard {
        // Press and release Shift — invisible to most apps
        std::thread::sleep(Duration::from_millis(rng.gen_range(100..300)));
        let _ = enigo.key(Key::Shift, Direction::Press);
        std::thread::sleep(Duration::from_millis(rng.gen_range(30..80)));
        let _ = enigo.key(Key::Shift, Direction::Release);
    }
}
