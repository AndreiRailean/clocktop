mod font;
use font::FONT;

use chrono::{Local, Timelike};
use clap::{Parser, ValueEnum};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use log::{debug, info};
use simplelog::{Config as LogConfig, LevelFilter, WriteLogger};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

use serde::Deserialize;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BlinkInterval {
    Hour,
    Half,
    Quarter,
    Minute,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum AppMode {
    Clock,
    Countdown,
    Stopwatch,
}

#[derive(PartialEq, Debug)]
enum TimerState {
    Running,
    Paused,
    Finished,
}

#[derive(PartialEq, Debug)]
enum StopwatchState {
    Idle,
    Running,
    Paused,
}

#[derive(Deserialize, Default, Debug)]
struct Config {
    blink: Option<BlinkInterval>,
    default_timer: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "clocktop", version, about = "Terminal clock widget")]
struct Cli {
    #[arg(short, long, value_enum)]
    blink: Option<BlinkInterval>,

    #[arg(short, long, num_args(0..=1))]
    timer: Option<String>,
}

fn get_config_path() -> PathBuf {
    let mut path = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        });
    path.push("clocktop");
    path.push("config.toml");
    path
}

fn load_config() -> Config {
    let path = get_config_path();
    debug!("Looking for config file at: {:?}", path);

    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str(&&content) {
                return config;
            }
        }
    }
    info!("No configuration file found or failed to parse. Using defaults.");
    Config::default()
}

fn parse_timer_string(s: &str) -> u64 {
    duration_str::parse(s)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn format_stopwatch_duration(elapsed: Duration, force_hours: bool) -> String {
    let total_secs = elapsed.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = elapsed.subsec_millis();

    if force_hours || hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    } else {
        format!("{:02}:{:02}.{:03}", minutes, seconds, millis)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn main() -> io::Result<()> {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };

    #[cfg(debug_assertions)]
    let _ = WriteLogger::init(
        log_level,
        LogConfig::default(),
        File::create("clocktop.log")?,
    );

    info!("Starting clocktop application...");

    let cli = Cli::parse();
    let config = load_config();

    let chosen_blink = cli.blink.or(config.blink);

    let mut app_mode = AppMode::Clock;
    let mut timer_state = TimerState::Paused;

    let raw_timer_str = cli
        .timer
        .or(config.default_timer)
        .unwrap_or_else(|| "25m".to_string());

    let initial_duration_secs = parse_timer_string(&raw_timer_str);
    let mut remaining_secs = initial_duration_secs;

    if std::env::args().any(|arg| arg == "-t" || arg == "--timer") {
        if initial_duration_secs > 0 {
            app_mode = AppMode::Countdown;
            timer_state = TimerState::Running;
            debug!(
                "Timer activated on launch. Value: {} seconds",
                initial_duration_secs
            );
        }
    };

    let mut stopwatch_state = StopwatchState::Idle;
    let mut stopwatch_elapsed = Duration::ZERO;
    let mut stopwatch_last_start: Option<Instant> = None;
    let mut stopwatch_laps: Vec<Duration> = Vec::new();
    let mut total_elapsed_at_last_lap = Duration::ZERO;

    let mut show_laps_overlay = false;
    let mut overlay_scroll_offset = 0;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut last_tick = Instant::now();

    loop {
        let now_instant = Instant::now();
        if now_instant.duration_since(last_tick) >= Duration::from_secs(1) {
            if timer_state == TimerState::Running {
                if remaining_secs > 0 {
                    remaining_secs -= 1;
                } else {
                    timer_state = TimerState::Finished;
                    info!("Timer reached zero! Alerting user.");
                }
            }
            last_tick = now_instant
        }

        let current_stopwatch_display_time = if stopwatch_state == StopwatchState::Running {
            if let Some(start_time) = stopwatch_last_start {
                stopwatch_elapsed + now_instant.duration_since(start_time)
            } else {
                stopwatch_elapsed
            }
        } else {
            stopwatch_elapsed
        };

        let mut mode_menu_buffer = String::new();

        terminal.draw(|frame| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            let clock_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(5),
                    Constraint::Min(0),
                ])
                .split(frame.area());

            let now = Local::now();
            let mut text_color = Color::Gray;

            let display_str = match app_mode {
                AppMode::Clock => {
                    let minute = now.minute();
                    let second = now.second();
                    let nano = now.nanosecond();
                    let milli = nano / 1_000_000;

                    let is_in_blink_window = match chosen_blink {
                        Some(BlinkInterval::Hour) => minute == 0 && second == 0,
                        Some(BlinkInterval::Half) => (minute == 0 || minute == 30) && second == 0,
                        Some(BlinkInterval::Quarter) => minute.is_multiple_of(15) && second == 0,
                        Some(BlinkInterval::Minute) => minute.is_multiple_of(1) && second == 0,
                        None => false,
                    };

                    let should_hide =
                        is_in_blink_window && matches!(&milli, 150..=250 | 450..=650 | 750..=950);

                    let should_hide_separator = !(200..800).contains(&milli);

                    if should_hide {
                        "".to_string()
                    } else {
                        if should_hide_separator {
                            now.format("%H %M %S").to_string()
                        } else {
                            now.format("%H:%M:%S").to_string()
                        }
                    }
                }
                AppMode::Countdown => {
                    let hours = remaining_secs / 3600;
                    let minutes = (remaining_secs % 3600) / 60;
                    let seconds = remaining_secs % 60;

                    if timer_state == TimerState::Finished {
                        text_color = Color::Red;
                        let milli = now.nanosecond() / 1_000_000;
                        if milli < 500 {
                            "00:00:00".to_string()
                        } else {
                            "".to_string()
                        }
                    } else if timer_state == TimerState::Paused {
                        text_color = Color::Yellow;
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                    } else {
                        text_color = Color::LightGreen;
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                    }
                }
                AppMode::Stopwatch => {
                    match stopwatch_state {
                        StopwatchState::Idle => text_color = Color::DarkGray,
                        StopwatchState::Paused => text_color = Color::Yellow,
                        StopwatchState::Running => text_color = Color::LightCyan,
                    }
                    format_stopwatch_duration(current_stopwatch_display_time, false)
                }
            };

            let mut spans_row = Vec::new();
            let mut current_idx = 0;

            while current_idx < display_str.len() {
                let next_char = &display_str[current_idx..current_idx + 1];
                let is_separator = next_char == ":" || next_char == ".";
                let len = 1;

                let sub_str = &display_str[current_idx..current_idx + len];
                let mut lines = vec![String::new(); 5];
                for ch in sub_str.chars() {
                    if let Some((_, pattern)) = FONT.iter().find(|(c, _)| **c == ch) {
                        for row in 0..5 {
                            lines[row].push_str(pattern[row]);
                            lines[row].push_str(" ");
                        }
                    }
                }

                spans_row.push((lines, is_separator));
                current_idx += len;
            }

            let mut final_lines: Vec<Line> = Vec::new();
            for row in 0..5 {
                let mut row_spans = Vec::new();
                for (lines, is_sep) in &spans_row {
                    let color = if *is_sep { Color::DarkGray } else { text_color };
                    row_spans.push(Span::styled(lines[row].clone(), Style::default().fg(color)));
                }
                final_lines.push(Line::from(row_spans));
            }
            let clock_widget = Paragraph::new(final_lines)
                .alignment(Alignment::Center)
                .style(Style::default().fg(text_color));
            frame.render_widget(clock_widget, clock_chunks[1]);

            if app_mode == AppMode::Stopwatch && !stopwatch_laps.is_empty() {
                let mut lap_lines = Vec::new();
                // Pick the last 3 lap milestones recorded to fit cleanly into terminal layouts
                let start_idx = stopwatch_laps.len().saturating_sub(3);
                for (i, lap) in stopwatch_laps.iter().enumerate().skip(start_idx) {
                    lap_lines.push(Line::from(vec![
                        Span::styled(
                            format!("Lap {:02}: ", i + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format_stopwatch_duration(*lap, true),
                            Style::default().fg(Color::Reset),
                        ),
                    ]));
                }
                let lap_widget = Paragraph::new(lap_lines).alignment(Alignment::Center);
                frame.render_widget(lap_widget, main_chunks[1]);
            }

            mode_menu_buffer.clear();
            let mut needs_sep = false;
            match app_mode {
                AppMode::Clock => {
                    if timer_state == TimerState::Running {
                        let hours = remaining_secs / 3600;
                        let minutes = (remaining_secs % 3600) / 60;
                        let seconds = remaining_secs % 60;
                        let _ = write!(
                            mode_menu_buffer,
                            "Timer Running: {:02}:{:02}:{:02}",
                            hours, minutes, seconds
                        );
                        needs_sep = true;
                    }

                    if stopwatch_state == StopwatchState::Running {
                        if needs_sep {
                            mode_menu_buffer.push_str(" | ");
                        }
                        mode_menu_buffer.push_str("Stopwatch Running");
                    }
                }
                AppMode::Countdown => match timer_state {
                    TimerState::Running => mode_menu_buffer.push_str("Pause: <space> | Reset: r"),
                    TimerState::Paused => mode_menu_buffer.push_str("Resume: <space> | Reset: r"),
                    _ => mode_menu_buffer.push_str("Reset: r"),
                },
                AppMode::Stopwatch => match stopwatch_state {
                    StopwatchState::Idle => {
                        mode_menu_buffer.push_str("Start: <space>");
                    }
                    StopwatchState::Running => mode_menu_buffer.push_str("Pause: <space> | Lap: l"),
                    StopwatchState::Paused => {
                        mode_menu_buffer.push_str("Resume: <space> | Reset: r");
                        if !stopwatch_laps.is_empty() {
                            mode_menu_buffer.push_str(" | View All Laps: <enter>");
                        }
                    }
                },
            }

            let mode_menu_widget = Paragraph::new(mode_menu_buffer)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(mode_menu_widget, main_chunks[2]);

            let main_menu = "Clock: 1 | Timer: 2 | Stopwatch: 3 | Quit: q".to_string();
            let main_menu_widget = Paragraph::new(main_menu)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(main_menu_widget, main_chunks[3]);

            if app_mode == AppMode::Stopwatch && show_laps_overlay {
                let area = centered_rect(60, 70, frame.area());
                // Clear out background pixels underneath the overlay box boundary
                frame.render_widget(Clear, area);
                let mut overlay_lines = Vec::new();
                for (i, lap) in stopwatch_laps.iter().enumerate() {
                    overlay_lines.push(Line::from(vec![
                        Span::styled(
                            format!("  Lap {:02}:    ", i + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format_stopwatch_duration(*lap, true),
                            Style::default().fg(Color::LightCyan),
                        ),
                    ]));
                }
                if stopwatch_laps.is_empty() {
                    overlay_lines.push(Line::from(Span::styled(
                        "  No laps recorded yet.",
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                let border_height_cost = 2;
                let visible_rows = area.height.saturating_sub(border_height_cost) as usize;
                let hint_title = if stopwatch_laps.len() > visible_rows {
                    " Complete Lap History [▲/▼ or j/k to Scroll] (Press [Enter] to Close) "
                } else {
                    " Complete Lap History (Press [Enter] to Close) "
                };

                let overlay_block = Block::default()
                    .title(Span::styled(
                        hint_title,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray));

                let overlay_widget = Paragraph::new(overlay_lines)
                    .block(overlay_block)
                    .scroll((overlay_scroll_offset as u16, 0))
                    .wrap(Wrap { trim: true });

                frame.render_widget(overlay_widget, area);

                if stopwatch_laps.len() > visible_rows {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("▲"))
                        .end_symbol(Some("▼"))
                        .track_symbol(Some("│"))
                        .thumb_symbol("█")
                        .style(Style::default().fg(Color::DarkGray));
                    let mut scrollbar_state =
                        ScrollbarState::new(stopwatch_laps.len().saturating_sub(visible_rows))
                            .position(overlay_scroll_offset);
                    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
                }
            }
        })?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            if key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL))
            {
                info!("Exit request received. Shutting down cleanly.");
                break;
            }

            if show_laps_overlay {
                let term_size = terminal.size()?;
                let visible_rows = (term_size.height * 70 / 100).saturating_sub(2) as usize;

                let max_scroll_limit = stopwatch_laps.len().saturating_sub(visible_rows);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        overlay_scroll_offset = overlay_scroll_offset.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if overlay_scroll_offset < max_scroll_limit {
                            overlay_scroll_offset += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        show_laps_overlay = false;
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('1') => app_mode = AppMode::Clock,
                KeyCode::Char('2') => app_mode = AppMode::Countdown,
                KeyCode::Char('3') => app_mode = AppMode::Stopwatch,
                KeyCode::Enter => {
                    if app_mode == AppMode::Stopwatch
                        && (stopwatch_state == StopwatchState::Paused
                            || stopwatch_state == StopwatchState::Idle)
                    {
                        show_laps_overlay = !show_laps_overlay;
                    }
                }
                KeyCode::Char(' ') => match app_mode {
                    AppMode::Countdown => {
                        timer_state = match timer_state {
                            TimerState::Running => TimerState::Paused,
                            TimerState::Paused => TimerState::Running,
                            TimerState::Finished => TimerState::Finished,
                        };
                    }
                    AppMode::Stopwatch => match stopwatch_state {
                        StopwatchState::Idle => {
                            stopwatch_state = StopwatchState::Running;
                            stopwatch_last_start = Some(Instant::now());
                        }
                        StopwatchState::Running => {
                            stopwatch_state = StopwatchState::Paused;
                            if let Some(start_time) = stopwatch_last_start {
                                stopwatch_elapsed += now_instant.duration_since(start_time);
                            }
                            stopwatch_last_start = None;
                        }
                        StopwatchState::Paused => {
                            show_laps_overlay = false;
                            stopwatch_state = StopwatchState::Running;
                            stopwatch_last_start = Some(Instant::now());
                        }
                    },
                    _ => {}
                },
                KeyCode::Char('l') => {
                    if app_mode == AppMode::Stopwatch && stopwatch_state == StopwatchState::Running
                    {
                        let current_lap_duration =
                            current_stopwatch_display_time - total_elapsed_at_last_lap;
                        stopwatch_laps.push(current_lap_duration);
                        total_elapsed_at_last_lap = current_stopwatch_display_time;
                    }
                }
                KeyCode::Char('r') => match app_mode {
                    AppMode::Countdown => {
                        remaining_secs = initial_duration_secs;
                        timer_state = if initial_duration_secs > 0 {
                            TimerState::Running
                        } else {
                            TimerState::Paused
                        }
                    }
                    AppMode::Stopwatch => {
                        if stopwatch_state == StopwatchState::Paused {
                            stopwatch_state = StopwatchState::Idle;
                            stopwatch_elapsed = Duration::ZERO;
                            stopwatch_last_start = None;
                            stopwatch_laps.clear();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
