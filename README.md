# break-time

[![Actions Status](https://github.com/cdepillabout/break-time/workflows/CI/badge.svg)](https://github.com/cdepillabout/break-time/actions)
[![crates.io](https://img.shields.io/crates/v/break-time.svg)](https://crates.io/crates/break-time)
[![dependency status](https://deps.rs/repo/github/cdepillabout/break-time/status.svg)](https://deps.rs/repo/github/cdepillabout/break-time)
![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)

break-time is an application that forces you to take breaks while working at
your computer.  This is convenient for people that want to avoid sitting for
too long, or staring at the computer screen for too long.

The main feature of break-time is that it is really hard to end a break
prematurely, but there are plugins provided to intelligently avoid breaks at
inconvenient times.  For instance, there is a plugin to avoid having a break
occur during a time when you have an event on your Google Calendar, as well as
a plugin to avoid a break when you are on a video chat in Google Meet.

## Installing

Installing with `cargo`:

```console
$ cargo install break-time
```

You'll need to have GTK libraries available in your environment for this to
work.

## Usage

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
