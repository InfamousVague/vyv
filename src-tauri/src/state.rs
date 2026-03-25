use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AwakeMode {
    Off,
    SystemOnly,
    DisplayAndSystem,
}

impl std::fmt::Display for AwakeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwakeMode::Off => write!(f, "Off"),
            AwakeMode::SystemOnly => write!(f, "System Only"),
            AwakeMode::DisplayAndSystem => write!(f, "Display + System"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerPreset {
    Min5,
    Min10,
    Min15,
    Min30,
    Min45,
    Hour1,
    Hour2,
    Hour4,
    Hour8,
    Indefinite,
}

impl TimerPreset {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            TimerPreset::Min5 => "5 minutes",
            TimerPreset::Min10 => "10 minutes",
            TimerPreset::Min15 => "15 minutes",
            TimerPreset::Min30 => "30 minutes",
            TimerPreset::Min45 => "45 minutes",
            TimerPreset::Hour1 => "1 hour",
            TimerPreset::Hour2 => "2 hours",
            TimerPreset::Hour4 => "4 hours",
            TimerPreset::Hour8 => "8 hours",
            TimerPreset::Indefinite => "Indefinite",
        }
    }

    pub fn duration_secs(&self) -> Option<u64> {
        match self {
            TimerPreset::Min5 => Some(5 * 60),
            TimerPreset::Min10 => Some(10 * 60),
            TimerPreset::Min15 => Some(15 * 60),
            TimerPreset::Min30 => Some(30 * 60),
            TimerPreset::Min45 => Some(45 * 60),
            TimerPreset::Hour1 => Some(60 * 60),
            TimerPreset::Hour2 => Some(2 * 60 * 60),
            TimerPreset::Hour4 => Some(4 * 60 * 60),
            TimerPreset::Hour8 => Some(8 * 60 * 60),
            TimerPreset::Indefinite => None,
        }
    }
}

pub struct TimerState {
    pub preset: TimerPreset,
    pub started_at: Instant,
    pub cancel: CancellationToken,
}

pub struct SimulationState {
    pub cancel: CancellationToken,
    pub profile_name: String,
}

pub struct AppState {
    pub awake_mode: Mutex<AwakeMode>,
    pub timer: Mutex<Option<TimerState>>,
    pub awake_guard: Mutex<Option<keepawake::KeepAwake>>,
    pub simulation: Mutex<Option<SimulationState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            awake_mode: Mutex::new(AwakeMode::Off),
            timer: Mutex::new(None),
            awake_guard: Mutex::new(None),
            simulation: Mutex::new(None),
        }
    }
}
