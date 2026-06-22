# Clocktop

`clocktop` is a terminal clock widget with multiple modes

```
❯ clocktop --help
clocktop 0.1.0

Terminal clock widget

Usage: clocktop [OPTIONS]

Options:
  -b, --blink <BLINK>        [possible values: hour, half, quarter, minute]
  -t, --timer [<TIMER>]
  -z, --timezone <TIMEZONE>
  -m, --mode <MODE>          [possible values: clock, timer, stopwatch, world]
  -h, --help                 Print help
  -V, --version              Print version

EXAMPLES:
  clocktop -t 45m                    Launch directly into a 45-minute countdown timer
  clocktop --timer 1h30s             Launch with a 1 hour and 30 seconds custom duration
  clocktop --mode world              Launch straight into the multi-city world clock panel
  clocktop -z America/New_York       Run the main clock displaying New York local time
  clocktop -m stopwatch -z UTC       Run the stopwatch and set timezone to UTC
```
## Operation
Once launched, use keys 1, 2, 3 and 4 to switch between modes as shown in the bottom menu.
1. Clock
2. Countdown timer
3. Stopwatch
4. World clock

If a mode has extra shortcuts, a submenu will be shown.

Using `-m` or `--mode` argument will start the app in one of the four modes `clock, timer, stopwatch, world`. If no mode is specified, the app launches into `clock` mode.

Clocktop defaults can be set in a config file.

## Time
<img width="920" height="516" alt="clocktop-time" src="https://github.com/user-attachments/assets/c6a4e2ae-f915-4c19-853e-73a8149b4d41" />

Default mode is clock. It blinks the separators and shows seconds. Timezone and date are shown in the header. Clock can be configured to blink on the hour, thirty minutes, each quarter or every minute. This can be done with either an argument (`clocktop --blink hour`) or via config file.

Pressing `1` will switch to showing time from any other mode.

By default, time is shown in computer's local time zone. City name is shown in the header. Override computer's local tz by specifying a `-z` or `--timezone` arg. When starting the app as `clocktop -z Australia/Perth` local time of the app is set to Perth. This argument also impacts the world clock mode.

## Countdown timer
<img width="920" height="516" alt="clocktop-timer" src="https://github.com/user-attachments/assets/c5c18e09-b5c8-4b7e-842c-2060bf2022eb" />

Timer counts down the time from the set value down to zero.

Timer mode is activated by either: 
- pressing a `2` key while in app
- by launching the app with `-m timer` argument
- by launching the app with `-t` or `--timer` arg with or without a value, i.e. `clocktop -t` or `clocktop -t 15s`

If timer value is not specified at launch, the app will try to use the value from the config file. If no value is found, the app defaults to 25 minutes.

After the countdown timer has been started, it is possible to switch to other modes. The app will flip back to timer when it reaches zero and flash big red zeros. In clock mode, if the countdown timer is running it will be shown in small text under the big clock.

Keyboard shortcuts for timer mode:
- `<space>` - pause/start
- `r` - reset and restart the timer whether it is already running or not

## Stopwatch
<img width="920" height="516" alt="clocktop-stopwatch" src="https://github.com/user-attachments/assets/7a615dc2-17dd-4d81-888b-650f49ee502f" />

Stopwatch allows timing of things with millisecond resolution. It starts by showing minutes, seconds and milliseconds. It will begin showing hours after 59 minutes. It doesn't track days. Lap tracking functionality is included.

Stopwatch mode is activated by:
- pressing `3` in the app
- launching with `-m stopwatch` argument

When the stopwatch is running it is possible to switch to another mode: the stopwatch will keep running and laps will be preserved.

Keyboard shortcuts for stopwatch mode:
- `<space>` - start/pause
- `r` - reset (only when paused)
- `l` - record a lap (only when running)
- `<enter>` - show/hide full laps list (when paused)

When tracking laps, the last 3 laps are shown under the big stopwatch display. Full list of laps can be openened by pressing `<enter>` when stopwatch is paused. Laps screen must be closed to switch to another mode or operate the sopwatch.

## World Clock
<img width="920" height="516" alt="clocktop-world" src="https://github.com/user-attachments/assets/4466ef53-3006-4459-9d9e-88fa9275fc87" />

World clock shows time in cities of the world. This mode requires setting up a config file or it defaults to 3 cities: New York, London and Tokio.

World clock shows city name, time difference, time and date in each city. Local timezone is always highlighted and displayed in bold. If local timezone is not listed in the config file, it will be injected at the top of the list. Each location shows its offset from the local time. When using `-z` or `--timezone` argument, clocktop will use it as its local timezone. This will impact main clock display and time difference calculations. Default time zone can also be set using the config file.

Each location indicates "daylight" or "business hours" by its colour. White/yellow colour indicates daylight, also a circle is shown. Blue shades indicate that location is outside of business hours. By default "daylight" is from 6am to 6pm, but this can be changed in config file.

## Config file
Config file is read from `$XDG_CONFIG_HOME/clocktop/config.toml`. 
If `$XDG_CONFIG_HOME` is not defined, we look for `$HOME/.config/clocktop/config.toml`

Example config file showing all settable options:
```toml
blink = "quarter" 
default_timer = "5s"
timezone = "Australia/Sydney"
world_clocks = [
	"America/Montreal", 
	"Australia/Perth", 
	"Australia/Brisbane",
]
daylight_start = 9 
daylight_end = 17
```

## Similar projects
Some other rust-based tui clocks on github
* [Clock TUI](https://github.com/race604/clock-tui)
* [tuime](https://github.com/nthnd/tuime)
