mod digits;
use digits::DIGITS;

use chrono::{Local, Timelike};
use clap::{Parser, ValueEnum};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use log::{debug, error, info};
use simplelog::{Config as LogConfig, LevelFilter, WriteLogger};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use serde::Deserialize;
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

#[derive(PartialEq)]
enum AppMode {
    Clock,
    Countdown,
}

#[derive(PartialEq)]
enum TimerState {
    Running,
    Paused,
    Finished,
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

        terminal.draw(|frame| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
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
            };

            let hour_str = display_str.get(0..2).unwrap_or("");
            let sep1_str = display_str.get(2..3).unwrap_or("");
            let min_str = display_str.get(3..5).unwrap_or("");
            let sep2_str = display_str.get(5..6).unwrap_or("");
            let sec_str = display_str.get(6..8).unwrap_or("");

            let build_block = |sub_str: &str| -> Vec<String> {
                let mut large_lines = vec![String::new(); 5];
                for ch in sub_str.chars() {
                    if let Some((_, pattern)) = DIGITS.iter().find(|(c, _)| **c == ch) {
                        for row in 0..5 {
                            large_lines[row].push_str(pattern[row]);
                            large_lines[row].push(' ');
                        }
                    }
                }
                large_lines
            };

            let hour_rows = build_block(hour_str);
            let sep1_rows = build_block(sep1_str);
            let min_rows = build_block(min_str);
            let sep2_rows = build_block(sep2_str);
            let sec_rows = build_block(sec_str);

            let mut final_lines: Vec<Line> = Vec::new();
            for row in 0..5 {
                final_lines.push(Line::from(vec![
                    Span::styled(hour_rows[row].clone(), Style::default().fg(text_color)),
                    Span::styled(sep1_rows[row].clone(), Style::default().fg(Color::DarkGray)),
                    Span::styled(min_rows[row].clone(), Style::default().fg(text_color)),
                    Span::styled(sep2_rows[row].clone(), Style::default().fg(Color::DarkGray)),
                    Span::styled(sec_rows[row].clone(), Style::default().fg(text_color)),
                ]));
            }

            let clock_widget = Paragraph::new(final_lines)
                .alignment(Alignment::Center)
                .style(Style::default().fg(text_color));
            frame.render_widget(clock_widget, clock_chunks[1]);

            let mut runtime_status = String::new();
            if timer_state == TimerState::Running && app_mode == AppMode::Clock {
                let hours = remaining_secs / 3600;
                let minutes = (remaining_secs % 3600) / 60;
                let seconds = remaining_secs % 60;
                runtime_status = format!(" ({:02}:{:02}:{:02})", hours, minutes, seconds);
            }

            let hint_text = match app_mode {
                AppMode::Clock => format!("Timer: c{} | Quit: q", runtime_status),

                AppMode::Countdown => match timer_state {
                    TimerState::Running => {
                        "Pause: <space> | Reset: r | Clock: c | Quit: q".to_string()
                    }
                    TimerState::Paused => {
                        "Resume: <space> | Reset: r | Clock: c | Quit: q".to_string()
                    }
                    TimerState::Finished => "Reset: r | Clock: c | Quit: q".to_string(),
                },
            };

            let hint_widget = Paragraph::new(hint_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(hint_widget, main_chunks[1]);
        })?;

        if event::poll(Duration::from_millis(50))?
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

            match key.code {
                KeyCode::Char('c') => {
                    app_mode = match app_mode {
                        AppMode::Clock => AppMode::Countdown,
                        AppMode::Countdown => AppMode::Clock,
                    }
                }
                KeyCode::Char(' ') => {
                    if app_mode == AppMode::Countdown {
                        timer_state = match timer_state {
                            TimerState::Running => TimerState::Paused,
                            TimerState::Paused => TimerState::Running,
                            TimerState::Finished => TimerState::Finished,
                        };
                    }
                }
                KeyCode::Char('r') => {
                    if app_mode == AppMode::Countdown {
                        remaining_secs = initial_duration_secs;
                        timer_state = if initial_duration_secs > 0 {
                            TimerState::Running
                        } else {
                            TimerState::Paused
                        };
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
