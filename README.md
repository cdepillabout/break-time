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
when you are on a video chat in Google Meet, Zoom, etc.

break-time currently only runs on Linux with X11.  However, PRs are welcome
adding support for other platforms and window systems.

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
configuration file.  Immediately kill it with <kbd>Ctrl</kbd>-<kbd>C</kbd> after
running it.

```console
$ break-time
^C
```

break-time should create a configuration file in
`~/.config/break-time/config.toml`.  Open up this configuration file in a text
editor to see what options are available.  If the explanations for any options
are not understandable, please open an issue.

The most interesting option will probably be `accounts` (or
`plugin.google_calendar.accounts`).  This is described in the next section.

After you have configured break-time, run it again.

```console
$ break-time
```

break-time will countdown until it is time for the next break.

break-time will create a systray icon.  If you mouse over it, it will tell you
how many minutes are left until your next break.  If you right click on the
systray icon, you can pause and resume the break countdown timer.

When it is time for your next break, break-time will pop up a screen telling
you to take a break.  You won't be able to close this screen until either the
break-time is over, or you press the spacebar 400 times.

### Plugins

break-time has plugins that are used to prevent a break from occurring.  Right
before a break is about to occur, break-time queries all the plugins and asks
if it is really okay to start a break.  This section explains how the plugins
work and how to configure them.

#### X Window Titles (Video Chat)

The X Window Title plugin checks whether or not there is an X Window with a
given name and title.

This is convenient to stop a break from occurring when you're in a video chat.

Currently, this plugin only checks for a few specific window names and titles.
You can see what window names and titles are checked for in
[this function](https://github.com/cdepillabout/break-time/blob/master/src/scheduler/plugins/window_titles.rs#L159-L193).

If you want additional applications to be supported, please open an
issue or send a PR modifying this function.

One way to check if this plugin is working is start break-time with a short
break interval, and then open https://meet.google.com/ in Firefox or Chromium.
break-time should not start a break while https://meet.google.com/ is open (and
the currently focused tab).

#### Google Calendar

The Google Calendar plugin checks whether or not there is an event on your
Google Calendar currently occurring.  If there is an event, the plugin stops
break-time from starting a break.

This is convenient to stop a break from happening when you're in a meeting at
work.

This plugin has a feature where if your Google Calendar event has a description
with the magic string `ignore break-time`, break-time will ignore it when
considering whether or not to start a break.

This plugin requires some configuration before it can be used.  First, you must
add your Gmail address to the break-time configuration file,
`~/.config/break-time/config.toml`.  The `accounts` (or
`plugin.google_calendar.accounts`) field should be set to a list of your Gmail
addresses.  I suggest only adding one at a time.

If your Gmail address is `example@gmail.com`, the configuration file will look
like the following:

```toml
[plugin.google_calendar]
accounts = [ "example@gmail.com", ]
```

After adding this, restart break-time.  break-time should output a message like
the following:

```console
$ break-time
Please direct your browser to https://accounts.google.com/o/oauth2/auth... and follow the instructions displayed there.
```

Make sure you are signed in to `example@gmail.com` in your browser (and not
another email address). Follow the link.  You should be presented with a
"Choose an account" page (as long as you have multiple Gmail accounts).  Make
sure you choose `example@gmail.com`.

Next, you'll be presented with a warning that the break-time app is not
verified.  Click the "Advanced" link, and then "Go to Break Time (unsafe)".

(This is currently necessary because I haven't completed the verification
request from Google for use of the Google Calendar API.  See issue
[#11](https://github.com/cdepillabout/break-time/issues/11) for slightly more
information.)

The next screen will ask you to grant break-time the following two permissions:

-   View events on all your calendars

    This allows break-time to check the start and end times of the events on
    your calendar.

-   View your calendars

    This allows break-time to determine what calendars you have, in order to
    determine where to check for events.

Once you grant break-time these permissions, break-time will save an OAuth
token to disk in `~/.cache/break-time/google-calendar/`.  This will allow
break-time to continue checking your Google Calendar without having to go
through this authorization step again in the future.

Note that break-time doesn't store your calendar data on disk, and it
definitely doesn't transmit it over the network after receiving it from the
Google Calendar API.  If you're worried about using this plugin, I'd recommend
reading the source code of the
[`google_calendar.rs`](https://github.com/cdepillabout/break-time/blob/master/src/scheduler/plugins/google_calendar.rs).
The only security concern is that break-time stores an OAuth token to disk so
that it can re-use it next time you start break-time.

If you want to use break-time with multiple Google Calendar accounts, you can
add multiple addresses to `plugin.google_calendar.accounts`.  Although, I suggest
only adding an account and doing authorization one at a time.

One way to check if this plugin is working is to set a short break interval,
and create an event on your calendar.  break-time should not start a break
while an event is taking place.

## Why

I noticed I was sitting in front of my computer for excessively long periods of
time.  I thought this was starting to lead to neck and shoulder pain, so I
wanted to find a way to force myself to take more breaks.

I tried a couple solutions, including a manual kitchen timer, timers on my
phone, applications like [Stretchly](https://hovancik.net/stretchly/), etc.
However, I've never found anything that really worked well.

I have three main problems with other solutions:

-   It is too easy to ignore the breaks.  When using a timer on my phone, I
	would often turn off the alarm and continue working.  When using Stretchly,
    I would often exit out of it whenever a break started.

    break-time fixes this by making it really hard to skip breaks.  Once a
    break starts, you're forced to step away from your computer.

-   Breaks occur at inconvenient times.  You never want a break to occur when
    you're doing something important, like in a meeting or in a video chat.

    break-time fixes this by having plugins to detect when you're in a meeting
    or in a video chat.

-   There is no warning that a break is coming.  With other solutions, it is
    frustrating when a break suddenly occurs in the middle of your work.

    break-time fixes this by having a countdown timer in the systray icon.
    You get a five-minute heads-up to wrap up any code you're writing,
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
