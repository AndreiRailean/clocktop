use chrono::{Local, Timelike};
use clap::{Parser, ValueEnum};
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

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum BlinkInterval {
    Hourly,
    Min30,
    Min15,
    Min,
}

#[derive(Parser, Debug)]
#[command(name = "clocktop", version, about = "A scaling terminal clock")]
struct Cli {
    #[arg(short, long, value_enum)]
    blink: Option<BlinkInterval>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

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

            let now = Local::now();
            let minute = now.minute();
            let second = now.second();
            let nano = now.nanosecond();
            let milli = nano / 1_000_000;

            let is_in_blink_window = match cli.blink {
                Some(BlinkInterval::Hourly) => minute == 0 && second == 0,
                Some(BlinkInterval::Min30) => (minute == 0 || minute == 30) && second == 0,
                Some(BlinkInterval::Min15) => minute.is_multiple_of(15) && second == 0,
                Some(BlinkInterval::Min) => minute.is_multiple_of(1) && second == 0,
                None => false,
            };

            let should_hide = is_in_blink_window
                && matches!(&milli, 100..=149 | 200..=249 | 500..=549 | 600..=649 | 700..=749 | 800..=849);
            //((250..499).contains(&milli) || milli >= 750);

            let large_clock_text = if should_hide {
                "\n\n\n\n".to_string()
            } else {
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
                large_lines.join("\n")
            };

            let clock_widget = Paragraph::new(large_clock_text).alignment(Alignment::Center);
            frame.render_widget(clock_widget, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(50))?
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
