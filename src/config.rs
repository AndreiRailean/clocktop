use crate::cli::Cli;
use crate::types;

use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Deserialize, Serialize, Debug)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blink: Option<types::BlinkInterval>,

    #[serde(rename = "default_timer")]
    #[serde(with = "humantime_serde")]
    pub timer: Duration,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    pub world_clocks: Vec<String>,

    pub daylight_start: u32,
    pub daylight_end: u32,

    #[serde(rename = "mode")]
    pub app_mode: types::AppMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blink: None,
            timer: Duration::from_secs(90),
            timezone: None,
            world_clocks: vec![
                "America/New_York".to_string(),
                "Europe/London".to_string(),
                "Asia/Tokyo".to_string(),
            ],
            daylight_start: 6,
            daylight_end: 18,
            app_mode: types::AppMode::Clock,
        }
    }
}

pub fn get_config_path() -> PathBuf {
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

impl AppConfig {
    pub fn try_load(cli_args: &Cli) -> Result<Self, figment::Error> {
        let config_file_path = get_config_path();
        Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Toml::file(&config_file_path))
            .merge(Serialized::defaults(&cli_args))
            .extract::<Self>()
    }
}
