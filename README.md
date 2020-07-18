# break-time

[![Actions Status](https://github.com/cdepillabout/break-time/workflows/CI/badge.svg)](https://github.com/cdepillabout/break-time/actions)
[![crates.io](https://img.shields.io/crates/v/break-time.svg)](https://crates.io/crates/break-time)
[![dependency status](https://deps.rs/repo/github/cdepillabout/break-time/status.svg)](https://deps.rs/repo/github/cdepillabout/break-time)
![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)

break-time is an application that forces you to take breaks while working at
your computer.  This is convenient for people that want to avoid sitting for
too long, or staring at the computer screen for too long.

The main feature of break-time is that there is no easy way to end a break
early.  Once a break starts, you are forced to stop using your computer.
However, there are plugins provided to avoid breaks at inconvenient times.  For
instance, there is a plugin to avoid having a break occur during a time when
you have an event on your Google Calendar, as well as a plugin to avoid a break
when you are on a video chat in Google Meet.

## Installing

break-time requires a few system libraries to be available.  On Debian/Ubuntu systems, these can be installed with the following command:

```console
$ sudo apt-get install libgtk-3-dev libxcb-screensaver0-dev
```

After this, you can install break-time with
[`cargo`](https://doc.rust-lang.org/cargo/):

```console
$ cargo install break-time
```

## Usage

Once break-time is installed, I suggest running it once so that it creates a
config file.  Immediately kill it with <kbd>Ctrl</kbd><kbd>C</kbd> after
running it.

```console
$ break-time
^C
```

break-time should create a config file in `~/.config/break-time/config.toml`.
Open up this config file in a text editor to see what options are available to
configure.  If any options are not understandable, please open an issue.

The most interesting option will probably be `accounts` (or
`plugin.google_calendar.accounts`).  This is described in the next section.

After you have configured break-time, run it again.

```console
$ break-time
```

break-time will count down until it is time for the next break.

break-time will create a systray icon.  If you mouse over it, it will tell you
how many minutes are left until your next break.  If you right click on the
systray icon, you can pause and resume the break count-down timer.

When it is time for your next break, break-time will pop up a screen telling
you to take a break.  You won't be able to close this screen until either the
break-time is over, or you press the spacebar 400 times.

## Why

I noticed that I was sitting in front of my computer for excessively long
periods of time.  I thought that this was starting to lead to neck and shoulder
pain, so I wanted to find a way to force myself to take more breaks.

I tried a couple solutions, including a manual kitchen timer, timers on my
phone, [Stretchly](https://hovancik.net/stretchly/), etc.  However,
I've never found anything that really worked well.

I have three main problems with other solutions:

-   It is too easy to ignore the breaks.  When using a timer on my phone, I
    would quite often just turn off the alarm and continue working.  When using
    Stretchly, I would often just immediately exit out of it whenever a break
    started.

    break-time fixes this by making it really hard to skip breaks.  Once a
    break starts, you're basically forced to step away from your computer.

-   Breaks occur at unconvenient times.  You never want a break to occur when
    you're doing something important, like in a meeting or doing a video chat.

    break-time fixes this by having plugins to detect when you're in a meeting
    or on a video chat.

-   There is no warning that a break is coming.  With other solutions, it is
    frustrating when a break suddenly occurs in the middle of your work.

    break-time fixes this by having a count-down timer in the systray icon.
    You get a five-minute heads-up to wrap up any functions you're writing,
    messages you're writing on Slack, documentation you're reading, etc.

break-time is one of the first non-trivial Rust programs I've created,
currently at around 2,500 lines (including whitespace and comments).

The main difficulty in writing break-time was trying to find a good abstraction
to deal with all the concurrent code.  In the end, I don't feel like I was able
to do this, but I created
[an issue](https://github.com/cdepillabout/break-time/issues/5) to think of how
to improve this.

## Contributions

Feel free to open an issue or PR for any
bugs, problems, suggestions, or improvements.
