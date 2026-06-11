use chrono::Local;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::Paragraph,
};
use std::io::{self, stdout};
use std::time::Duration;

mod digits;
use digits::DIGITS;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(5),
                    Constraint::Min(0),
                ])
                .split(frame.area());

            let time_str = Local::now().format("%H:%M:%S").to_string();

            let mut large_lines = vec![String::new(); 5];

            for ch in time_str.chars() {
                if let Some((_, pattern)) = DIGITS.iter().find(|(c, _)| **c == ch) {
                    for row in 0..5 {
                        large_lines[row].push_str(pattern[row]);
                        large_lines[row].push(' ');
                    }
                }
            }

            let large_clock_text = large_lines.join("\n");

            let clock_widget = Paragraph::new(large_clock_text).alignment(Alignment::Center);

            frame.render_widget(clock_widget, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && (key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)))
        {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
