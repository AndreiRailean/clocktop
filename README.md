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
