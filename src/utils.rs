use chrono_tz::Tz;
use std::str::FromStr;
use std::time::Duration;

pub fn resolve_timezone(input_tz: &str) -> Tz {
    if !input_tz.trim().is_empty()
        && let Ok(parsed_tz) = Tz::from_str(input_tz)
    {
        return parsed_tz;
    }

    if let Ok(local_tz_str) = iana_time_zone::get_timezone()
        && let Ok(local_tz) = Tz::from_str(&local_tz_str)
    {
        return local_tz;
    }

    chrono_tz::UTC
}

/// Returns the display city name for a timezone, e.g. `Tz::America__New_York` → `"New York"`.
pub fn tz_city_name(tz: Tz) -> String {
    let zone_name = format!("{:?}", tz);
    zone_name
        .split('/')
        .next_back()
        .unwrap_or(&zone_name)
        .replace('_', " ")
}

pub fn format_stopwatch_duration(elapsed: Duration, force_hours: bool) -> String {
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

pub fn print_config_error(err: figment::Error) {
    eprintln!("\n\x1b[1;31mValidation Error:\x1b[0m Failed to process properties.");
    for e in err {
        if let Some(metadata) = &e.metadata
            && let Some(figment::Source::File(path)) = &metadata.source
        {
            eprintln!("  \x1b[1mFile:\x1b[0m {}", path.display());
        }
        if !e.path.is_empty() {
            eprintln!("  \x1b[1mSetting:\x1b[0m {}", e.path.join("."));
        }
        let friendly_msg = e
            .to_string()
            .replace("invalid type:", "Invalid type:")
            .replace("invalid value:", "Invalid value:");
        eprintln!("  \x1b[1mProblem:\x1b[0m {}\n", friendly_msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_stopwatch_duration() {
        assert_eq!(
            format_stopwatch_duration(Duration::from_millis(123), false),
            "00:00.123"
        );
        assert_eq!(
            format_stopwatch_duration(Duration::from_secs(65) + Duration::from_millis(500), false),
            "01:05.500"
        );
        assert_eq!(
            format_stopwatch_duration(Duration::from_secs(3600) + Duration::from_millis(1), false),
            "01:00:00.001"
        );
        assert_eq!(
            format_stopwatch_duration(Duration::from_secs(65), true),
            "00:01:05.000"
        );
    }

    #[test]
    fn test_resolve_timezone() {
        assert_eq!(resolve_timezone("UTC"), chrono_tz::UTC);
        assert_eq!(
            resolve_timezone("America/New_York"),
            chrono_tz::America::New_York
        );
        // Fallback checks (system local or UTC if none found)
        let resolved = resolve_timezone("Invalid/Timezone");
        // Must resolve to some timezone, typically UTC or system local Tz
        assert!(!resolved.name().is_empty());
    }

    #[test]
    fn test_tz_city_name() {
        // Underscores replaced with spaces, only the city part after the last slash
        assert_eq!(tz_city_name(chrono_tz::America::New_York), "New York");
        assert_eq!(tz_city_name(chrono_tz::Australia::Lord_Howe), "Lord Howe");

        // Single-component zone — no slash, whole name used as-is
        assert_eq!(tz_city_name(chrono_tz::UTC), "UTC");

        // Zone whose city segment has no underscores
        assert_eq!(tz_city_name(chrono_tz::Europe::London), "London");
    }
}
