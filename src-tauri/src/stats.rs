use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UptimeStats {
    pub total_seconds: u64,
    pub milestones_seen: Vec<String>,
}

impl Default for UptimeStats {
    fn default() -> Self {
        Self {
            total_seconds: 0,
            milestones_seen: Vec::new(),
        }
    }
}

pub struct StatsTracker {
    pub stats: Mutex<UptimeStats>,
    pub session_start: Mutex<Option<Instant>>,
}

impl StatsTracker {
    pub fn new(persisted: Option<UptimeStats>) -> Self {
        Self {
            stats: Mutex::new(persisted.unwrap_or_default()),
            session_start: Mutex::new(None),
        }
    }

    pub fn start_session(&self) {
        let mut start = self.session_start.lock().unwrap();
        *start = Some(Instant::now());
    }

    pub fn end_session(&self) {
        let mut start = self.session_start.lock().unwrap();
        if let Some(started) = start.take() {
            let elapsed = started.elapsed().as_secs();
            let mut stats = self.stats.lock().unwrap();
            stats.total_seconds += elapsed;

            // Check milestones
            let total_hours = stats.total_seconds / 3600;
            let new_milestones = check_milestones(total_hours, &stats.milestones_seen);
            stats.milestones_seen.extend(new_milestones);
        }
    }

    pub fn get_stats(&self) -> UptimeStats {
        let mut stats = self.stats.lock().unwrap().clone();

        // Add current session time if active
        let start = self.session_start.lock().unwrap();
        if let Some(started) = *start {
            stats.total_seconds += started.elapsed().as_secs();
        }

        stats
    }

    pub fn format_total(&self) -> String {
        let stats = self.get_stats();
        let hours = stats.total_seconds / 3600;
        let mins = (stats.total_seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }

    pub fn get_dosage_level(&self) -> &'static str {
        let stats = self.get_stats();
        let hours = stats.total_seconds / 3600;

        match hours {
            0 => "First timer",
            1..=23 => "Casual user",
            24..=99 => "Regular dose",
            100..=499 => "Heavy usage",
            500..=999 => "Prescription strength",
            _ => "Heroic dose",
        }
    }
}

fn check_milestones(total_hours: u64, seen: &[String]) -> Vec<String> {
    let milestones = [
        (1, "First Hour"),
        (24, "24 Hours"),
        (100, "100 Hours"),
        (168, "1 Week"),
        (500, "500 Hours"),
        (720, "1 Month"),
        (1000, "1,000 Hours — Seek help"),
    ];

    milestones
        .iter()
        .filter(|(hours, _)| total_hours >= *hours)
        .filter(|(_, name)| !seen.contains(&name.to_string()))
        .map(|(_, name)| name.to_string())
        .collect()
}
