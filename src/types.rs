use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BlinkInterval {
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
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ModeArg {
    Clock,
    Timer,
    Stopwatch,
    World,
}

#[derive(PartialEq, Debug)]
pub enum TimerState {
    Running,
    Paused,
    Finished,
}

#[derive(PartialEq, Debug)]
pub enum StopwatchState {
    Idle,
    Running,
    Paused,
}
