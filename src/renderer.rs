use crate::appstate;
use crate::font::FONT;
use crate::types::{AppMode, BlinkInterval, StopwatchState, TimerState};
use crate::utils;

use chrono::{Offset, Timelike, Utc};
use chrono_tz::Tz;
use ratatui::widgets::ListItem;

use std::fmt::Write;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table,
    },
};

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

#[derive(Default, Debug)]
pub struct Renderer {
    /// Reused across frames to avoid a heap allocation every tick at 60 fps.
    /// Cleared at the start of each render pass; `clear()` retains capacity.
    mode_menu_buffer: String,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, frame: &mut Frame, app_state: &appstate::AppState) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
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
            .split(main_chunks[1]);

        let zoned_now = &app_state.zoned_now();
        let hour = zoned_now.hour();
        let minute = zoned_now.minute();
        let second = zoned_now.second();
        let milli = zoned_now.nanosecond() / 1_000_000;

        let header_label = format!("{} ", utils::tz_city_name(app_state.active_tz).to_uppercase());
        let header_date = zoned_now.format("%a, %b %d, %Y").to_string();

        let mut text_color = Color::Gray;

        let header_line = match app_state.active_mode() {
            AppMode::Clock => Line::from(vec![
                Span::styled(header_label, Style::default().fg(Color::DarkGray)),
                Span::styled(header_date, Style::default().fg(Color::DarkGray)),
            ]),
            AppMode::World => {
                // Keep the World Table dashboard top completely clear since it uses first-class column headers
                Line::from("")
            }
            _ => Line::from(vec![Span::styled(
                format!(
                    "{}{}, {}",
                    header_label,
                    zoned_now.format("%H:%M"),
                    zoned_now.format("%a, %b %d")
                ),
                Style::default().fg(Color::DarkGray),
            )]),
        };

        frame.render_widget(
            Paragraph::new(header_line).alignment(Alignment::Center),
            main_chunks[0],
        );

        let display_str: Option<String> = match app_state.active_mode() {
            AppMode::Clock => {
                let is_in_blink_window = match app_state.clock().blink() {
                    Some(BlinkInterval::Hour) => minute == 0 && second == 0,
                    Some(BlinkInterval::Half) => (minute == 0 || minute == 30) && second == 0,
                    Some(BlinkInterval::Quarter) => minute.is_multiple_of(15) && second == 0,
                    Some(BlinkInterval::Minute) => second == 0,
                    None => false,
                };

                let should_hide = is_in_blink_window && matches!(&milli, 200..=400 | 600..=800);

                let should_hide_separator = !matches!(&milli, 200..=800);
                let separator = if should_hide_separator { ' ' } else { ':' };

                if should_hide {
                    Some("".to_string())
                } else {
                    Some(format!(
                        "{:02}{}{:02}{}{:02}",
                        hour, separator, minute, separator, second
                    ))
                }
            }

            AppMode::Countdown => {
                let (hours, minutes, seconds) = app_state.timer().remaining_time_parts();

                if app_state.timer().state() == TimerState::Finished {
                    text_color = Color::Red;
                    if milli <= 400 {
                        Some("00:00:00".to_string())
                    } else {
                        Some("".to_string())
                    }
                } else if app_state.timer().state() == TimerState::Paused {
                    text_color = Color::Yellow;
                    Some(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                } else {
                    text_color = Color::LightGreen;
                    Some(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                }
            }

            AppMode::Stopwatch => {
                match app_state.stopwatch().state() {
                    StopwatchState::Idle => text_color = Color::DarkGray,
                    StopwatchState::Paused => text_color = Color::Yellow,
                    StopwatchState::Running => text_color = Color::LightCyan,
                }
                Some(utils::format_stopwatch_duration(app_state.stopwatch().elapsed(), false))
            }

            AppMode::World => {
                self.draw_world_table(frame, app_state, main_chunks[1]);
                None
            }
        };

        if let Some(display_str) = display_str {
            let mut spans_row = Vec::new();
            for ch in display_str.chars() {
                let is_separator = ch == ':' || ch == '.';
                let mut lines = vec![String::new(); 5];
                if let Some((_, pattern)) = FONT.iter().find(|(c, _)| *c == ch) {
                    for row in 0..5 {
                        lines[row].push_str(pattern[row]);
                        lines[row].push(' ');
                    }
                }
                spans_row.push((lines, is_separator));
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
        }

        if app_state.active_mode() == AppMode::Stopwatch && !app_state.stopwatch().laps().is_empty()
        {
            let mut lap_lines = Vec::new();
            let start_idx = app_state.stopwatch().laps().len().saturating_sub(3);
            for (i, lap) in app_state
                .stopwatch()
                .laps()
                .iter()
                .enumerate()
                .skip(start_idx)
            {
                lap_lines.push(Line::from(vec![
                    Span::styled(
                        format!("Lap {:02}: ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        utils::format_stopwatch_duration(*lap, true),
                        Style::default().fg(Color::Reset),
                    ),
                ]));
            }
            let lap_widget = Paragraph::new(lap_lines).alignment(Alignment::Center);
            frame.render_widget(lap_widget, main_chunks[2]);
        }

        // Mode Menu
        self.mode_menu_buffer.clear();
        let mut needs_sep = false;
        match app_state.active_mode() {
            AppMode::Clock => {
                if app_state.timer().is_running() {
                    let (hours, minutes, seconds) = app_state.timer().remaining_time_parts();
                    if hours > 0 {
                        let _ = write!(
                            self.mode_menu_buffer,
                            "Timer Running: {:02}:{:02}:{:02}",
                            hours, minutes, seconds
                        );
                    } else {
                        let _ = write!(
                            self.mode_menu_buffer,
                            "Timer Running: {:02}:{:02}",
                            minutes, seconds
                        );
                    }
                    needs_sep = true;
                }

                if app_state.stopwatch().is_running() {
                    if needs_sep {
                        self.mode_menu_buffer.push_str(" | ");
                    }
                    let elapsed = app_state.stopwatch().elapsed();
                    let total_secs = elapsed.as_secs();
                    let hours = total_secs / 3600;
                    let minutes = (total_secs % 3600) / 60;
                    let seconds = total_secs % 60;
                    if hours > 0 {
                        let _ = write!(
                            self.mode_menu_buffer,
                            "Stopwatch Running: {:02}:{:02}:{:02}",
                            hours, minutes, seconds
                        );
                    } else {
                        let _ = write!(
                            self.mode_menu_buffer,
                            "Stopwatch Running: {:02}:{:02}",
                            minutes, seconds
                        );
                    }
                }
            }
            AppMode::Countdown => match app_state.timer().state() {
                TimerState::Running => self.mode_menu_buffer.push_str("Pause: <space> | Reset: r"),
                TimerState::Paused => self.mode_menu_buffer.push_str("Resume: <space> | Reset: r"),
                _ => self.mode_menu_buffer.push_str("Reset: r"),
            },
            AppMode::Stopwatch => match app_state.stopwatch().state() {
                StopwatchState::Idle => {
                    self.mode_menu_buffer.push_str("Start: <space>");
                }
                StopwatchState::Running => {
                    self.mode_menu_buffer.push_str("Pause: <space> | Lap: l")
                }
                StopwatchState::Paused => {
                    self.mode_menu_buffer.push_str("Resume: <space> | Reset: r");
                    if !app_state.stopwatch().laps().is_empty() {
                        self.mode_menu_buffer.push_str(" | View all laps: <enter>");
                    }
                }
            },
            _ => {}
        }

        let mode_menu_widget = Paragraph::new(self.mode_menu_buffer.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(mode_menu_widget, main_chunks[3]);

        // Main Menu
        let dim_style = Style::default().fg(Color::DarkGray);
        let active_style = Style::default()
            .fg(Color::Reset)
            .add_modifier(Modifier::BOLD);

        let main_menu_line = Line::from(vec![
            Span::styled(
                "Clock",
                if app_state.active_mode() == AppMode::Clock {
                    active_style
                } else {
                    dim_style
                },
            ),
            Span::styled(" 1", dim_style),
            Span::styled(" | ", dim_style),
            Span::styled(
                "Timer",
                if app_state.active_mode() == AppMode::Countdown {
                    active_style
                } else {
                    dim_style
                },
            ),
            Span::styled(" 2", dim_style),
            Span::styled(" | ", dim_style),
            Span::styled(
                "Stopwatch",
                if app_state.active_mode() == AppMode::Stopwatch {
                    active_style
                } else {
                    dim_style
                },
            ),
            Span::styled(" 3", dim_style),
            Span::styled(" | ", dim_style),
            Span::styled(
                "World",
                if app_state.active_mode() == AppMode::World {
                    active_style
                } else {
                    dim_style
                },
            ),
            Span::styled(" 4", dim_style),
            Span::styled(" | ", dim_style),
            Span::styled("?", dim_style),
        ]);

        let main_menu_widget = Paragraph::new(main_menu_line).alignment(Alignment::Center);
        frame.render_widget(main_menu_widget, main_chunks[4]);

        // Stopwatch Lap Overlay
        let sw = app_state.stopwatch();
        if app_state.active_mode() == AppMode::Stopwatch && sw.is_overlay_open() {
            let overlay_area = centered_rect(60, 70, frame.area());
            frame.render_widget(Clear, overlay_area);

            let overlay_lines: Vec<ListItem> = sw
                .laps()
                .iter()
                .enumerate()
                .map(|(i, lap)| {
                    let time_str = utils::format_stopwatch_duration(*lap, true);

                    let lap_row = Line::from(vec![
                        Span::styled("Lap ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{:02}", i + 1),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" │ ", Style::default().fg(Color::Gray)),
                        Span::styled(time_str, Style::default().fg(Color::Cyan)),
                    ]);

                    ListItem::new(lap_row)
                })
                .collect();

            let mut list_state = ListState::default().with_selected(Some(sw.scroll_index()));

            let hint_title = " Lap History (Press [Enter] to Close) ";

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

            let overlay_widget = List::new(overlay_lines)
                .block(overlay_block)
                .highlight_symbol(">> ");

            frame.render_stateful_widget(overlay_widget, overlay_area, &mut list_state);

            let content_length = sw.laps().len();
            let border_height_cost = 2;
            let viewport_height = overlay_area.height.saturating_sub(border_height_cost) as usize;

            if content_length > viewport_height {
                let mut scroll_state =
                    ScrollbarState::new(content_length).position(sw.scroll_index()); // Mirror our abstract selection index

                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█")
                    .style(Style::default().fg(Color::DarkGray));

                // Render the scrollbar on top of the list's right-hand border track area
                frame.render_stateful_widget(scrollbar, overlay_area, &mut scroll_state);
            }
        }
        if app_state.show_help {
            self.draw_help_overlay(frame, app_state);
        }
    }

    fn draw_world_table(&self, frame: &mut Frame, app_state: &appstate::AppState, area: Rect) {
        let column_widths = [
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(35),
        ];

        let header_row = Row::new(vec![
            Cell::from("  LOCATION").style(Style::default().fg(Color::DarkGray)),
            Cell::from("").style(Style::default().fg(Color::DarkGray)),
            Cell::from("  TIME").style(Style::default().fg(Color::DarkGray)),
        ])
        .height(1);

        let mut data_rows = Vec::new();
        let mut tracked_zones = app_state.world_clocks().clone();

        let baseline_str = format!("{:?}", &app_state.active_tz).replace("::", "/");
        if !tracked_zones.contains(&baseline_str) {
            tracked_zones.insert(0, baseline_str.clone());
        }

        for zone_str in tracked_zones {
            let target_tz: Tz = match zone_str.parse::<Tz>() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let z_now = Utc::now().with_timezone(&target_tz);
            let base_now = Utc::now().with_timezone(&app_state.active_tz);

            let clean_city = utils::tz_city_name(target_tz).to_uppercase();

            let base_offset_secs = base_now.offset().fix().local_minus_utc();
            let target_offset_secs = z_now.offset().fix().local_minus_utc();
            let diff_secs = target_offset_secs - base_offset_secs;
            let diff_hours = diff_secs / 3600;

            let diff_str = if diff_hours == 0 {
                "".to_string()
            } else if diff_hours > 0 {
                format!("+{}h", diff_hours)
            } else {
                format!("{}h", diff_hours)
            };

            let is_primary = target_tz == app_state.active_tz;
            let current_hour = z_now.hour();
            let daylight_start_hour = app_state.daylight_start;
            let daylight_end_hour = app_state.daylight_end;

            let is_daylight = if daylight_start_hour <= daylight_end_hour {
                current_hour >= daylight_start_hour && current_hour < daylight_end_hour
            } else {
                // Handles overnight shifts gracefully if someone sets e.g. start=22, end=6
                current_hour >= daylight_start_hour || current_hour < daylight_end_hour
            };

            let main_color = if is_daylight {
                if is_primary { Color::Yellow } else { Color::White }
            } else {
                if is_primary { Color::Cyan } else { Color::LightCyan }
            };

            let (dot_char, dot_color) = if is_daylight {
                ("○ ", Color::Yellow)
            } else {
                ("  ", Color::Blue)
            };

            let mut city_style = Style::default().fg(main_color);
            if is_primary {
                city_style = city_style.add_modifier(Modifier::BOLD);
            }
            let diff_style = Style::default().fg(Color::DarkGray);

            let time_cell_content = Line::from(vec![
                Span::styled(dot_char, Style::default().fg(dot_color)),
                Span::styled(
                    z_now.format("%H:%M ").to_string(),
                    Style::default().fg(main_color),
                ),
                Span::styled(
                    z_now.format("%a, %b %d").to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            data_rows.push(
                Row::new(vec![
                    Cell::from(format!("  {}", clean_city)).style(city_style),
                    Cell::from(diff_str).style(diff_style),
                    Cell::from(time_cell_content),
                ])
                .height(1),
            );
        }

        let world_table = Table::new(data_rows, column_widths)
            .header(header_row)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default());

        let current_width = frame.area().width;
        let table_area = if current_width > 80 {
            // If terminal window is wide, center it within an 80-column block
            let horizontal_padding = (current_width.saturating_sub(80)) / 2;
            let layout_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(horizontal_padding),
                    Constraint::Length(80),
                    Constraint::Length(horizontal_padding),
                ])
                .split(area);
            layout_split[1]
        } else {
            area
        };

        frame.render_widget(world_table, table_area);
    }

    pub fn draw_help_overlay(&self, frame: &mut Frame, state: &appstate::AppState) {
        if !state.show_help {
            return;
        }

        // 1. Center a floating popup layout on screen over any existing graphics
        let overlay_area = centered_rect(80, 80, frame.area());

        // 2. Fetch data rows and convert them to rich styled elements
        let manifest = HELP_MANIFEST;
        let items: Vec<ListItem> = manifest
            .iter()
            .map(|row| {
                if row.is_header {
                    let line = Line::from(vec![Span::styled(
                        row.description,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )]);
                    ListItem::new(line)
                } else {
                    let line = Line::from(vec![
                        Span::styled(
                            format!(" {} ", row.shortcut),
                            Style::default()
                                .bg(Color::DarkGray)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" ── ", Style::default().fg(Color::DarkGray)),
                        Span::styled(row.description, Style::default().fg(Color::Gray)),
                    ]);
                    ListItem::new(line)
                }
            })
            .collect();

        // 3. Inject our selection index to jump focus to the right section
        let mut list_state = ListState::default().with_selected(Some(state.help_scroll_index));

        let help_list = List::new(items)
            .block(
                Block::default()
                    .title(" Shortcut menu (Press ? to Close) ")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40))) // Subtle row backdrop anchor
            .highlight_symbol("→ ");

        // 4. Paint to target overlay layout
        frame.render_widget(Clear, overlay_area); // This wipes out background grid cells cleanly
        frame.render_stateful_widget(help_list, overlay_area, &mut list_state);

        // 5. Build and draw tracking scrollbar indicators
        let content_length = manifest.len();
        let viewport_height = overlay_area.height.saturating_sub(2) as usize;
        if content_length > viewport_height {
            let mut scroll_state =
                ScrollbarState::new(content_length).position(state.help_scroll_index);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Style::default().fg(Color::Yellow));
            frame.render_stateful_widget(scrollbar, overlay_area, &mut scroll_state);
        }
    }
}

#[allow(dead_code)]
pub struct HelpRow {
    is_header: bool,
    shortcut: &'static str,
    description: &'static str,
    category: &'static str,
}

pub const HELP_MANIFEST: &[HelpRow] = &[

        // --- GLOBAL / CLOCK MODE ---
        HelpRow {
            is_header: true,
            shortcut: "",
            description: "--- Global & Clock Controls ---",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: " Esc ",
            description: "Cycle view mode panels forward",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  ?  ",
            description: "Toggle this interactive help panel",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  q  ",
            description: "Quit",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  1  ",
            description: "Clock mode",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  2  ",
            description: "Timer mode",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  3  ",
            description: "Stopwatch mode",
            category: "Global",
        },
        HelpRow {
            is_header: false,
            shortcut: "  4  ",
            description: "World Clock mode",
            category: "Global",
        },
        // --- TIMER MODE ---
        HelpRow {
            is_header: true,
            shortcut: "",
            description: "--- Timer Controls ---",
            category: "Timer",
        },
        HelpRow {
            is_header: false,
            shortcut: "Space",
            description: "Toggle timer (Start/Pause)",
            category: "Timer",
        },
        HelpRow {
            is_header: false,
            shortcut: "  r  ",
            description: "Reset and restart timer",
            category: "Timer",
        },
        // --- STOPWATCH MODE ---
        HelpRow {
            is_header: true,
            shortcut: "",
            description: "--- Stopwatch Controls ---",
            category: "Stopwatch",
        },
        HelpRow {
            is_header: false,
            shortcut: "Space",
            description: "Toggle stopwatch ticker (Start/Pause)",
            category: "Stopwatch",
        },
        HelpRow {
            is_header: false,
            shortcut: "  l  ",
            description: "Record split lap fragment duration",
            category: "Stopwatch",
        },
        HelpRow {
            is_header: false,
            shortcut: "  r  ",
            description: "Reset stopwatch accumulator values to zero",
            category: "Stopwatch",
        },
        HelpRow {
            is_header: false,
            shortcut: "Enter",
            description: "Toggle historic lap overlay viewer",
            category: "Stopwatch",
        },
];
