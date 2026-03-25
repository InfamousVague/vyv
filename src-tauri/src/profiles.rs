#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimProfile {
    Slack,
    Teams,
    Zoom,
    Custom(CustomProfile),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomProfile {
    pub name: String,
    pub mouse_movement: bool,
    pub keyboard_input: bool,
    pub interval_min_ms: u64,
    pub interval_max_ms: u64,
}

impl SimProfile {
    pub fn label(&self) -> &str {
        match self {
            SimProfile::Slack => "Slack",
            SimProfile::Teams => "Teams",
            SimProfile::Zoom => "Zoom",
            SimProfile::Custom(p) => &p.name,
        }
    }

    pub fn mouse_movement(&self) -> bool {
        match self {
            SimProfile::Slack => true,
            SimProfile::Teams => true,
            SimProfile::Zoom => true,
            SimProfile::Custom(p) => p.mouse_movement,
        }
    }

    pub fn keyboard_input(&self) -> bool {
        match self {
            SimProfile::Slack => false,
            SimProfile::Teams => true,
            SimProfile::Zoom => false,
            SimProfile::Custom(p) => p.keyboard_input,
        }
    }

    /// Returns (min_ms, max_ms) interval between actions
    pub fn interval_range(&self) -> (u64, u64) {
        match self {
            SimProfile::Slack => (60_000, 120_000),     // 60-120s
            SimProfile::Teams => (45_000, 90_000),      // 45-90s
            SimProfile::Zoom => (30_000, 60_000),       // 30-60s
            SimProfile::Custom(p) => (p.interval_min_ms, p.interval_max_ms),
        }
    }

    pub fn builtin_profiles() -> Vec<SimProfile> {
        vec![SimProfile::Slack, SimProfile::Teams, SimProfile::Zoom]
    }
}
