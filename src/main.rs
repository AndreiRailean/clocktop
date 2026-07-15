mod font;

mod appstate;
mod cli;
mod config;
mod renderer;
mod types;
mod utils;

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::renderer::Renderer;
use crate::types::AppMode;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

use std::io::{self, stdout};
use std::process;
use std::time::Instant;

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

fn main() -> io::Result<()> {
    let cli_args = cli::Cli::load();

    if cli_args.command == Some(cli::Commands::Validate) {
        let path = config::get_config_path();
        println!(
            "\x1b[1mValidating configuration file:\x1b[0m {}",
            path.display()
        );

        match AppConfig::try_load(&cli_args) {
            Ok(_) => {
                println!("\x1b[1;32m✓\x1b[0m Configuration file syntax is valid.");
                process::exit(0);
            }
            Err(err) => {
                utils::print_config_error(err);
                process::exit(1);
            }
        }
    }

    let config = match AppConfig::try_load(&cli_args) {
        Ok(cfg) => cfg,
        Err(err) => {
            utils::print_config_error(err);
            process::exit(1);
        }
    };

    let mut app_state = AppState::new_from_config(&config);
    let mut renderer = Renderer::new();

    if std::env::args().any(|arg| arg == "-t" || arg == "--timer")
        && !app_state.timer().duration().is_zero()
    {
        app_state.set_active_mode(AppMode::Countdown);
        app_state.update_timer(|timer| {
            timer.reset();
        });
    };

    // Restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let _guard = RawModeGuard::new()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut last_tick = Instant::now();

    loop {
        let tick_rate = app_state.tick_rate();

        // Keyboard input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        let is_input_event = event::poll(timeout)?;

        if is_input_event && let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                break;
            }
            if key.code == KeyCode::Char('?') {
                app_state.toggle_help();
                continue;
            }

            if app_state.show_help {
                // GATED MODAL SHORTCUTS: Scroll through help items
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        app_state.help_scroll_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app_state.help_scroll_down();
                    }
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                        app_state.toggle_help();
                    }
                    _ => {}
                }
                continue;
            }
            // Mode-specific keys
            match app_state.active_mode() {
                AppMode::Countdown => match key.code {
                    KeyCode::Char(' ') => {
                        app_state.update_timer(|timer| {
                            timer.toggle();
                        });
                    }
                    KeyCode::Char('r') => {
                        app_state.update_timer(|timer| {
                            timer.reset();
                        });
                    }
                    _ => {}
                },
                AppMode::Stopwatch => {
                    if app_state.stopwatch().is_overlay_open() {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                app_state.update_stopwatch(|sw| {
                                    sw.scroll_up();
                                });
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app_state.update_stopwatch(|sw| {
                                    sw.scroll_down();
                                });
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                app_state.update_stopwatch(|sw| {
                                    sw.toggle_overlay();
                                });
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Enter => {
                            app_state.update_stopwatch(|sw| {
                                sw.toggle_overlay();
                            });
                        }
                        KeyCode::Char(' ') => {
                            app_state.update_stopwatch(|sw| {
                                sw.toggle();
                            });
                        }
                        KeyCode::Char('r') => {
                            app_state.update_stopwatch(|sw| {
                                sw.reset();
                            });
                        }
                        KeyCode::Char('l') => {
                            app_state.update_stopwatch(|sw| {
                                sw.record_lap();
                            });
                        }
                        _ => {}
                    };
                }
                _ => {}
            }

            // Global Keys
            match key.code {
                KeyCode::Char('1') => app_state.set_active_mode(AppMode::Clock),
                KeyCode::Char('2') => app_state.set_active_mode(AppMode::Countdown),
                KeyCode::Char('3') => app_state.set_active_mode(AppMode::Stopwatch),
                KeyCode::Char('4') => app_state.set_active_mode(AppMode::World),
                _ => {}
            }
        }

        let now = Instant::now();
        if is_input_event || last_tick.elapsed() >= tick_rate {
            if last_tick.elapsed() >= tick_rate {
                last_tick = now;
            }
            app_state.tick(now, chrono::Utc::now());
        }

        let _ = terminal.draw(|frame| renderer.render(frame, &app_state));
    }

    Ok(())
}
