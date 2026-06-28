use std::time::Duration;

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
        if let Some(metadata) = &e.metadata {
            if let Some(figment::Source::File(path)) = &metadata.source {
                eprintln!("  \x1b[1mFile:\x1b[0m {}", path.display());
            }
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
