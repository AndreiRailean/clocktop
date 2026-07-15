use chrono::DateTime;
use chrono::Utc;
use chrono_tz::Tz;
use chrono_tz::UTC;

use crate::config::AppConfig;
use crate::renderer;
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
}

impl TimerModeState {
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
                self.state = TimerState::Finished;

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

    // UI vars
    overlay_open: bool,
    scroll_index: usize,
}

impl StopwatchModeState {
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

    pub fn toggle_overlay(&mut self) {
        if self.is_running() {
            return;
        }

        self.overlay_open = !self.overlay_open;
        // Reset scroll position when opening/closing
        if !self.overlay_open {
            self.scroll_index = 0;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.overlay_open && self.scroll_index > 0 {
            self.scroll_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.overlay_open && !self.laps.is_empty() && self.scroll_index < self.laps.len() - 1 {
            self.scroll_index += 1;
        }
    }

    pub fn is_overlay_open(&self) -> bool {
        self.overlay_open
    }
    pub fn scroll_index(&self) -> usize {
        self.scroll_index
    }
}

#[derive(Debug)]
pub struct AppState {
    active_mode: AppMode,

    pub clock: ClockModeState,
    timer: TimerModeState,
    stopwatch: StopwatchModeState,
    pub active_tz: chrono_tz::Tz,

    world_clocks: Vec<String>,
    pub daylight_start: u32,
    pub daylight_end: u32,

    last_tick: Instant,
    base_now_clock: DateTime<Utc>,

    pub show_help: bool,
    pub help_scroll_index: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let default_config = AppConfig::default();

        Self {
            active_mode: default_config.app_mode,
            active_tz: UTC,
            last_tick: Instant::now(),
            base_now_clock: Utc::now(),

            world_clocks: default_config.world_clocks,
            daylight_start: default_config.daylight_start,
            daylight_end: default_config.daylight_end,

            stopwatch: StopwatchModeState::default(),
            timer: TimerModeState::default(),
            clock: ClockModeState::default(),

            show_help: false,
            help_scroll_index: 0,
        }
    }
}

impl AppState {
    pub fn new_from_config(config: &AppConfig) -> Self {
        let mut state = AppState {
            active_mode: config.app_mode,
            active_tz: utils::resolve_timezone(config.timezone.as_deref().unwrap_or("")),

            // WORLD
            world_clocks: config.world_clocks.clone(),
            ..Default::default()
        };

        state.clock.blink = config.blink;

        state.timer.set_duration(config.timer);

        state
    }

    pub fn tick(&mut self, now_instant: Instant, now_clock: DateTime<Utc>) {
        self.base_now_clock = now_clock;
        let delta = now_instant.duration_since(self.last_tick);

        if self.timer.is_running()
            && let TimerTickResult::TimerExpired = self.timer.tick(delta)
        {
            self.active_mode = AppMode::Countdown;
        }

        if self.stopwatch.is_running() {
            self.stopwatch.tick(delta);
        }
        self.last_tick = now_instant;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll_index = match self.active_mode {
                AppMode::Clock | AppMode::World => 0,
                AppMode::Countdown => 9,
                AppMode::Stopwatch => 12,
            }
        }
    }

    pub fn help_scroll_up(&mut self) {
        if self.help_scroll_index > 0 {
            self.help_scroll_index -= 1;
        }
    }

    pub fn help_scroll_down(&mut self) {
        let max_idx = renderer::HELP_MANIFEST.len() - 1;
        if self.help_scroll_index < max_idx {
            self.help_scroll_index += 1;
        }
    }

    pub fn zoned_now(&self) -> DateTime<Tz> {
        self.base_now_clock.with_timezone(&self.active_tz)
    }

    pub fn set_active_mode(&mut self, mode: AppMode) {
        if self.active_mode != mode {
            self.active_mode = mode;
        }
    }

    pub fn active_mode(&self) -> AppMode {
        self.active_mode
    }

    pub fn world_clocks(&self) -> &Vec<String> {
        &self.world_clocks
    }

    pub fn tick_rate(&self) -> Duration {
        match self.active_mode {
            AppMode::Stopwatch if self.stopwatch().is_running() => Duration::from_millis(30),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{StopwatchState, TimerState};

    #[test]
    fn test_timer_mode_state() {
        let mut timer = TimerModeState::default();
        assert_eq!(timer.state(), TimerState::Paused);

        timer.set_duration(Duration::from_secs(10));
        assert_eq!(timer.state(), TimerState::Paused);
        assert_eq!(timer.remaining, Duration::from_secs(10));

        // Toggle to run
        timer.toggle();
        assert_eq!(timer.state(), TimerState::Running);

        // Tick 3 seconds
        let res = timer.tick(Duration::from_secs(3));
        assert_eq!(res, TimerTickResult::NoEvent);
        assert_eq!(timer.remaining, Duration::from_secs(7));

        // Tick remaining 7 seconds
        let res = timer.tick(Duration::from_secs(7));
        assert_eq!(res, TimerTickResult::TimerExpired);
        assert_eq!(timer.remaining, Duration::ZERO);
        assert_eq!(timer.state(), TimerState::Finished);

        // Toggle when finished
        timer.toggle();
        assert_eq!(timer.state(), TimerState::Finished);

        // Reset
        timer.reset();
        assert_eq!(timer.state(), TimerState::Running);
        assert_eq!(timer.remaining, Duration::from_secs(10));
    }

    #[test]
    fn test_stopwatch_mode_state() {
        let mut sw = StopwatchModeState::default();
        assert_eq!(*sw.state(), StopwatchState::Idle);

        sw.tick(Duration::from_secs(5));
        assert_eq!(sw.elapsed(), Duration::ZERO); // Shouldn't tick when not running

        sw.toggle();
        assert_eq!(*sw.state(), StopwatchState::Running);

        sw.tick(Duration::from_secs(5));
        assert_eq!(sw.elapsed(), Duration::from_secs(5));

        sw.record_lap();
        assert_eq!(sw.laps(), &[Duration::from_secs(5)]);

        sw.tick(Duration::from_secs(3));
        assert_eq!(sw.current_lap_time(), Duration::from_secs(3));
        assert_eq!(sw.elapsed(), Duration::from_secs(8));

        sw.toggle();
        assert_eq!(*sw.state(), StopwatchState::Paused);

        sw.reset();
        assert_eq!(*sw.state(), StopwatchState::Idle);
        assert_eq!(sw.elapsed(), Duration::ZERO);
    }
}
