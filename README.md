# Clocktop

`clocktop` is a terminal clock widget. It is built for running in a terminal or tmux pane and supports multiple modes. You can run multiple instances of `clocktop` to display the information you want or switch between modes within one instance:.

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

If a mode has extra shortcuts, you'll see a submenu.

Using `-m` or `--mode` argument will start the app in one of the four modes `clock, timer, stopwatch, world`. If no mode is specified, the app launches into `clock` mode.

## Time
<img width="920" height="516" alt="clocktop-time" src="https://github.com/user-attachments/assets/c6a4e2ae-f915-4c19-853e-73a8149b4d41" />

Default mode is clock. It blinks the separators and shows seconds. Timezone and date are shown in the header. Clock can be configured to blink on the hour, thirty minutes, each quarter or every minute. You can do this with either an argument (`clocktop --blink hour`) or via config file.

You can switch to this mode from any other mode by pressing `1`

## Countdown timer
<img width="920" height="516" alt="clocktop-timer" src="https://github.com/user-attachments/assets/c5c18e09-b5c8-4b7e-842c-2060bf2022eb" />

Timer counts down the time from the set value down to zero.

Timer mode is activated by either: 
- pressing a `2` key while in app
- by launching the app with `-m timer` argument
- by launching the app with `-t` or `--timer` arg with or without a value, i.e. `clocktop -t` or `clocktop -t 15s`

If you do not specify a timer value at launch, the app will try to use the value from the config file. If no value is found, the app defaults to 25 minutes and this value cannot be changed while the app is running.

When you launch the countdown timer, you can switch to other modes and the app will flip back to timer when it reaches zero and you'll see big red zeros flashing at you. In clock mode, you can see the countdown timer (if it is running) in small text under the big clock.

Keyboard shortcuts for timer mode:
- ` ` (`<space>`) will pause and restart the timer
- `r` will reset and restart the timer whether it is already running or not

## Stopwatch
<img width="920" height="516" alt="clocktop-stopwatch" src="https://github.com/user-attachments/assets/7a615dc2-17dd-4d81-888b-650f49ee502f" />

Stopwatch allows you to time things with millisecond resolution. When launched, you see minutes, seconds and milliseconds. It will start showing hours after 59 minutes. It doesn't track days. You also get lap tracking functionality with no limit to how many laps you can track.

Stopwatch is activated by:
- pressing `3` in the app
- launching with `-m stopwatch` argument

If the stopwatch is running and you switch to another mode, the stopwatch will not stop and will keep on running and laps are preserved.

Keyboard shortcuts for stopwatch mode:
- ` ` (`<space>`) starts and pauses the stopwatch
- `r` resets the stopwatch if it is paused
- `l` records a lap when stopwatch is running
- `<enter>` opens the laps list when stopwatch is paused. this also closes the laps list.

When tracking laps, you can see the last 3 laps under the main stopwatch indicator. When stopwatch is paused you can open the list of all laps by pressing `<enter>`. When in full laps screen you cannot switch to another mode or operate the sopwatch.

## World Clock
<img width="920" height="516" alt="clocktop-world" src="https://github.com/user-attachments/assets/4466ef53-3006-4459-9d9e-88fa9275fc87" />



