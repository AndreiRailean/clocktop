use chrono_tz::UTC;

use crate::config::AppConfig;
use crate::utils;

use crate::types::{AppMode, BlinkInterval, StopwatchState, TimerState};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct ClockModeState {
    blink: Option<BlinkInterval>,
}

impl ClockModeState {
    pub fn blink(&self) -> &Option<BlinkInterval> {
        &self.blink
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimerTickResult {
    NoEvent,
    TimerExpired,
}

#[derive(Debug, Default)]
pub struct TimerModeState {
    state: TimerState,
    duration: Duration,
    remaining: Duration,

    // TODO: consider moving to presentation logic
    pub alert_triggered: bool,
}

impl TimerModeState {
    pub fn set_state(&mut self, state: TimerState) {
        self.state = state;
    }

    pub fn state(&self) -> TimerState {
        self.state
    }

    pub fn toggle(&mut self) {
        self.state = match self.state {
            TimerState::Running => TimerState::Paused,
            TimerState::Paused => {
                if !self.remaining.is_zero() {
                    TimerState::Running
                } else {
                    TimerState::Paused
                }
            }
            _ => self.state,
        }
    }

    pub fn reset(&mut self) {
        self.alert_triggered = false;
        if !self.duration.is_zero() {
            self.state = TimerState::Running;
            self.remaining = self.duration;
        };
    }

    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
        self.remaining = duration;
    }

    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    pub fn remaining_time(&self) -> &Duration {
        &self.remaining
    }

    pub fn remaining_time_parts(&self) -> (u64, u64, u64) {
        let remaining_secs = self.remaining.as_secs();

        let hours = remaining_secs / 3600;
        let minutes = (remaining_secs % 3600) / 60;
        let seconds = remaining_secs % 60;

        (hours, minutes, seconds)
    }

    pub fn tick(&mut self, delta: Duration) -> TimerTickResult {
        if self.state != TimerState::Running || self.remaining.is_zero() {
            return TimerTickResult::NoEvent;
        }

        match self.remaining.checked_sub(delta) {
            Some(new_time) if !new_time.is_zero() => {
                self.remaining = new_time;
                TimerTickResult::NoEvent
            }
            _ => {
                self.remaining = Duration::ZERO;
                self.state = TimerState::Paused;

                TimerTickResult::TimerExpired
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct StopwatchModeState {
    state: StopwatchState,

    elapsed: Duration,
    laps: Vec<Duration>,
}

impl StopwatchModeState {
    pub fn set_state(&mut self, state: StopwatchState) {
        self.state = state;
    }

    pub fn toggle(&mut self) {
        self.state = match self.state {
            StopwatchState::Running => StopwatchState::Paused,
            StopwatchState::Paused => StopwatchState::Running,
            StopwatchState::Idle => StopwatchState::Running,
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        if !self.is_running() {
            return;
        }

        self.elapsed += delta;
    }

    pub fn record_lap(&mut self) {
        if !self.is_running() {
            return;
        }

        self.laps.push(self.current_lap_time());
    }

    pub fn current_lap_time(&self) -> Duration {
        let completed_laps_total: Duration = self.laps.iter().sum();

        self.elapsed
            .checked_sub(completed_laps_total)
            .unwrap_or_default()
    }

    pub fn laps(&self) -> &[Duration] {
        &self.laps
    }

    pub fn is_running(&self) -> bool {
        self.state == StopwatchState::Running
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn state(&self) -> &StopwatchState {
        &self.state
    }

    pub fn reset(&mut self) {
        if self.is_running() {
            return;
        }

        *self = Self::default();
    }

    pub fn pause(&mut self) {
        if !self.is_running() {
            return;
        }

        self.state = StopwatchState::Paused;
    }

    pub fn resume(&mut self) {
        if self.state != StopwatchState::Paused {
            return;
        }

        self.state = StopwatchState::Running;
    }
}

pub struct AppState {
    is_dirty: bool,
    active_mode: AppMode,

    pub clock: ClockModeState,
    timer: TimerModeState,
    stopwatch: StopwatchModeState,
    pub active_tz: chrono_tz::Tz,

    last_tick_time: Instant,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_mode: AppMode::Clock,
            is_dirty: true,
            active_tz: UTC,
            last_tick_time: Instant::now(),

            stopwatch: StopwatchModeState::default(),
            timer: TimerModeState::default(),
            clock: ClockModeState::default(),
        }
    }
}

impl AppState {
    pub fn new_from_config(config: &AppConfig) -> Self {
        let mut state = AppState::default();
        state.active_mode = config.app_mode;
        state.active_tz = utils::resolve_timezone(config.timezone.as_deref().unwrap_or(""));
        state.clock.blink = config.blink;

        // TIMER
        state.timer.set_duration(config.timer);

        // WORLD1

        state
    }

    pub fn tick(&mut self, delta: Duration) {
        match self.active_mode {
            AppMode::Countdown => {
                if let TimerTickResult::TimerExpired = self.timer.tick(delta) {
                    self.active_mode = AppMode::Countdown;
                    self.mark_dirty();
                }
            }
            AppMode::Stopwatch => {
                if self.stopwatch.is_running() {
                    self.mark_dirty();
                }
                self.stopwatch.tick(delta)
            }
            AppMode::Clock => {
                //
            }
            AppMode::World => {
                //
            }
        }
    }

    pub fn set_active_mode(&mut self, mode: AppMode) {
        if self.active_mode != mode {
            self.active_mode = mode;
            self.mark_dirty();
        }
    }

    pub fn active_mode(&self) -> AppMode {
        self.active_mode
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn take_dirty_flag(&mut self) -> bool {
        let dirty = self.is_dirty;
        self.is_dirty = false;
        dirty
    }

    pub fn tick_rate(&self) -> Duration {
        match self.active_mode {
            AppMode::Stopwatch => match self.stopwatch.state {
                StopwatchState::Running => Duration::from_millis(30),
                _ => Duration::from_millis(200),
            },
            _ => Duration::from_millis(200),
        }
    }

    //
    // CLOCK
    //
    pub fn clock(&self) -> &ClockModeState {
        &self.clock
    }

    //
    // STOPWATCH
    //
    pub fn stopwatch(&self) -> &StopwatchModeState {
        &self.stopwatch
    }

    pub fn update_stopwatch<F>(&mut self, update_fn: F)
    where
        F: FnOnce(&mut StopwatchModeState),
    {
        update_fn(&mut self.stopwatch);
        self.mark_dirty();
    }

    //
    // TIMER
    //
    pub fn timer(&self) -> &TimerModeState {
        &self.timer
    }

    pub fn update_timer<F>(&mut self, update_fn: F)
    where
        F: FnOnce(&mut TimerModeState),
    {
        update_fn(&mut self.timer);
        self.mark_dirty();
    }
}
