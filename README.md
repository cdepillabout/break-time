# break-time

[![Actions Status](https://github.com/cdepillabout/break-time/workflows/CI/badge.svg)](https://github.com/cdepillabout/break-time/actions)
[![crates.io](https://img.shields.io/crates/v/break-time.svg)](https://crates.io/crates/break-time)
[![dependency status](https://deps.rs/repo/github/cdepillabout/break-time/status.svg)](https://deps.rs/repo/github/cdepillabout/break-time)
![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)

break-time is an application that forces you to take breaks while working at
your computer.  This is convenient for people that want to avoid sitting for
too long, or staring at the computer screen for too long.

The main feature of break-time is that it is really hard to end a break
prematurely, but break-time provides plugins to intelligently avoid breaks at
inconvenient times.  For instance, there is a plugin to avoid having a break
occur during a time when you have an event on your Google Calendar, as well as
a plugin to avoid a break when you are doing a video chat in Google Meet.

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
smart phone, [Stretchly](https://hovancik.net/stretchly/), etc.  However,
I've never found anything that really stuck.

break-time is one of the first non-trivial Rust programs I've created,
currently at around 2,500 lines (including whitespace and comments).

## Contributions

Feel free to open an issue or PR for any
bugs/problems/suggestions/improvements.
