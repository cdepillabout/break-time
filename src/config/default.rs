use indoc::indoc;

pub const DEFAULT_CONFIG_SETTINGS: &str = indoc!(
    "
    # The number of seconds in a break.
    break_duration_seconds = 600 # 10 minutes

    # The number of seconds in between breaks.
    seconds_between_breaks = 3000 # 50 minutes

    # The number of seconds to use for a break if you've been idle. This
    # means that if break-time detects that you've been idle for 8 minutes,
    # it will use that time as a break.  It will then start waiting for
    # another seconds_between_breaks until starting another break.
    idle_detection_seconds = 480 # 8 minutes

    [plugin.google_calendar]
    # A list of strings, one for each Google account you want to authenticate with.
    accounts = []

    [plugin.x11_window_title_checker]
    "
);
