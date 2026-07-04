use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, ValueEnum, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BlinkInterval {
    #[default]
    Hour,
    Half,
    Quarter,
    Minute,
}

#[derive(Default, PartialEq, Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    #[default]
    Clock,
    Countdown,
    Stopwatch,
    World,
}

// command line argument for launching into a specific mode
#[derive(Debug, Default, Clone, Copy, ValueEnum, PartialEq)]
pub enum ModeArg {
    #[default]
    Clock,
    Timer,
    Stopwatch,
    World,
}

#[derive(Debug, Default, Clone, Copy, ValueEnum, PartialEq)]
pub enum TimerState {
    Running,
    #[default]
    Paused,
    Finished,
}

#[derive(Debug, Default, Clone, Copy, ValueEnum, PartialEq)]
pub enum StopwatchState {
    #[default]
    Idle,
    Running,
    Paused,
}
