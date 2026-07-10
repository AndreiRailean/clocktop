use crate::types::{AppMode, BlinkInterval};
use shadow_rs::shadow;

use clap::Parser;
use serde::Serialize;
use std::time::Duration;

shadow!(build);

#[derive(clap::Subcommand, Serialize, Clone, PartialEq, Debug)]
pub enum Commands {
    // Validate the configuration file syntax and exit
    Validate,
}

#[derive(Parser, Debug, Serialize)]
#[command(
    name = "clocktop",
    version,
    long_version = build::CLAP_LONG_VERSION,
    about = "Terminal clock widget",
    help_template = "\
{name} {version}
{author-with-newline}{about-section}
{usage-heading} {usage}

{all-args}

EXAMPLES:
  clocktop -t 45m                    Launch directly into a 45-minute countdown timer
  clocktop --timer 1h30s             Launch with a 1 hour and 30 seconds custom duration
  clocktop --mode world              Launch straight into the multi-city world clock panel
  clocktop -z America/New_York       Run the main clock displaying New York local time
  clocktop -m stopwatch -z UTC       Run the stopwatch and set timezone to UTC
"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    blink: Option<BlinkInterval>,

    #[arg(short, long, num_args(0..=1), value_parser = humantime::parse_duration)]
    #[serde(
        rename = "default_timer",
        with = "humantime_serde",
        skip_serializing_if = "Option::is_none"
    )]
    timer: Option<Duration>,

    #[arg(short = 'z', long = "timezone")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,

    #[arg(short = 'm', long = "mode", default_value = "clock")]
    mode: Option<AppMode>,
}

impl Cli {
    pub fn load() -> Self {
        let mut cli_args = Cli::parse();

        if cli_args.timer.is_some() {
            cli_args.mode = Some(AppMode::Countdown);
        }

        cli_args
    }
}
