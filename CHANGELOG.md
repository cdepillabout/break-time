
## next

*   Add a window title check for Zoom calls.
    [#17](https://github.com/cdepillabout/break-time/pull/17)

*   Add a window title check for Skype, Jitsi, and BigBlueButton.
    [#17](https://github.com/cdepillabout/break-time/pull/17)

*   For the Google Calendar plugin, ignore events from consideration where the
    event description contains the string `ignore break-time`.  This gives a
    nice way to make sure that break-time continues to force breaks to occur
    even if you have an event on your google calendar.
    [#19](https://github.com/cdepillabout/break-time/pull/19)

*   For the Google Calendar plugin, ignores events where the status is
    `cancelled`.  For some reason, sometimes events that become cancelled still
    show up on your calendar and in responses from the Google Calendar APIs.
    [#20](https://github.com/cdepillabout/break-time/pull/20)

    Also ignore events where the status is `needsAction`.  Even though you
    don't participate in an event, the event status stays `needsAction` and
    doesn't become `cancelled`.  This is worked around by also ignoring events
    where the status is `needsAction`.
    [#22](https://github.com/cdepillabout/break-time/pull/22)

*   For the Google Calendar plugin, treat all events as single events (instead of
    treating repeated events specially).  This fixes a bug with handling of
    repeated events.  Sometimes the Google Calendar API would return repeated
    events in times when it doesn't make sense, but this PR fixes this.
    [#23](https://github.com/cdepillabout/break-time/pull/23)

*   Add two command line commands: `google-calendar list-events` and
    `google-calendar ignore-event`.
    [#25](https://github.com/cdepillabout/break-time/pull/25)

    `google-calendar list-events` lists all the Google Calendar events that are
    occurring right now that will cause break-time to ignore a scheduled break
    time.

    ```console
    $ break-time google-calendar list-events
    me@my-email.com:
    - id: Some("123phhhlpppoohh88hhcl9988b"), summary: Some("Some test event")
    ```

    You can take these events and pass them to the `ignore-event` command to
    set a property on an event that will cause `break-time` to ignore it when
    it is trying to figure out whether or not it can break.

    ```console
    $ break-time google-calendar ignore-event "123phhhlpppoohh88hhcl9988b"
    ```

*   Add a value to the config file `idle_detection_enabled`.  It is set to
    `true` by default.  If you set it to false, then the X idle timer is
    ignored, so breaks will still occur even if you have stepped away from your
    computer. [#31](https://github.com/cdepillabout/break-time/pull/31)

## 0.1.2

*   Add a window title check for Slack calls.
    [#16](https://github.com/cdepillabout/break-time/pull/16)

## 0.1.1

*   Fix a bug where the idle detector panics on overflow from subtraction.
    [#12](https://github.com/cdepillabout/break-time/pull/12)

## 0.1.0

*   Initial release.
