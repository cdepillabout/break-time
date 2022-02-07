// This module defines an IdleDetector. It sends a message over a channel whenever it notices that
// the X Server has been idle for a certain amount of time.
//
// This is used to reset the break timer whenever the user has stepped away from the computer.
//
// This currently uses a simple loop while querying the screensaver info, but really we should be
// using a callback-based approach while listening to the XServer's XSync IDLETIME counter.  Here
// are a few examples of using this:
//
// - https://chromium.googlesource.com/chromiumos/platform/power_manager/+/refs/heads/0.12.433.B62/xidle.h
// - https://chromium.googlesource.com/chromiumos/platform/power_manager/+/refs/heads/0.12.433.B62/xidle.cc
// - https://chromium.googlesource.com/chromiumos/platform/power_manager/+/refs/heads/tx/xidle-example.cc
// - https://chromium.googlesource.com/chromiumos/platform/power_manager/+/refs/heads/0.12.433.B62/xidle_monitor.h
// - https://www.x.org/releases/X11R7.7/doc/xextproto/sync.html
// - https://github.com/freedesktop/xorg-xserver/blob/7f962c70b6d9c346477f23f6c15211e749110078/Xext/sync.c

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use super::InnerMsg;
use crate::config::Config;
use crate::prelude::*;

const SLEEP_SECONDS: u64 = 20;
const SLEEP_MILLISECONDS: u128 = (SLEEP_SECONDS as u128) * 1000;

pub struct IdleDetector {
    conn: xcb::Connection,
    root_window: xcb::Window,
    restart_wait_time_sender: Sender<InnerMsg>,
}

impl IdleDetector {
    pub fn new(restart_wait_time_sender: Sender<InnerMsg>) -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let setup: xcb::Setup = conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let preferred_screen_pos = usize::try_from(screen_num)
            .expect("x11 preferred_screen is not positive");
        let screen: xcb::Screen = roots.nth(preferred_screen_pos).unwrap();
        let root_window: xcb::Window = screen.root();

        Self {
            conn,
            root_window,
            restart_wait_time_sender,
        }
    }

    pub fn run(
        config: &Config,
        idle_detection_enabled: Arc<Mutex<bool>>,
        restart_wait_time_sender: Sender<InnerMsg>,
    ) -> ! {
        let idle_detector = Self::new(restart_wait_time_sender);
        let idle_detection_milliseconds =
            config.settings.idle_detection_seconds * 1000;
        loop {
            let time_before_sleep = SystemTime::now();

            let sleep_time_duration = Duration::from_secs(SLEEP_SECONDS);
            std::thread::sleep(sleep_time_duration);

            // Calculate the actual amount of time that has passed during sleep.
            // This will potentially be different from the sleep time because the computer could be
            // suspended during the above std::thread::sleep().
            let time_difference_milliseconds: u128 = time_before_sleep
                .elapsed()
                .unwrap_or(sleep_time_duration)
                .as_millis();

            // We subtract out the sleep time to get just the amount that the computer would have
            // been suspended for.  If the computer wasn't actually suspended, then this should be
            // very close to 0.
            let suspend_milliseconds: u128 =
                time_difference_milliseconds.saturating_sub(SLEEP_MILLISECONDS);

            let idle_query_res = xcb::screensaver::query_info(
                &idle_detector.conn,
                idle_detector.root_window,
            )
            .get_reply()
            .unwrap();

            let ms_since_user_input = idle_query_res.ms_since_user_input();

            println!(
                "idle detector: ms_since_user_input: {}, suspend_milliseconds: {}, idle_detection_milliseconds: {}",
                ms_since_user_input,
                suspend_milliseconds,
                idle_detection_milliseconds,
            );

            let use_idle_detection =
            {
                *idle_detection_enabled.lock().unwrap()
            };

            if has_been_idle(
                idle_detection_milliseconds.into(),
                ms_since_user_input.into(),
                suspend_milliseconds,
            ) {
                if use_idle_detection {
                    println!(
                        "idle detector detected that we have been idle, so sending HasBeenIdle message",
                    );
                    idle_detector
                        .restart_wait_time_sender
                        .send(InnerMsg::HasBeenIdle).expect("TODO: figure out what to do about channels potentially failing");
                }
                else {
                    println!(
                        "idle detector detected that we have been idle, but idle_detection is not enable, so not sending HasBeenIdle message",
                    );
                }
            }
        }
    }
}

const fn has_been_idle(
    idle_detection_milliseconds: u128,
    milliseconds_since_user_input: u128,
    suspend_milliseconds: u128,
) -> bool {
    (milliseconds_since_user_input + suspend_milliseconds)
        >= idle_detection_milliseconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_been_idle() {
        // This is a test where has_been_idle() should return true if the computer has been idle
        // for at least 20 seconds.  We've detected that there was been no user input in X for 10
        // seconds, and the computer has been asleep for 15 seconds, which adds up to 25 seconds
        // total.
        let res = has_been_idle(20000, 10000, 15000);

        assert_eq!(res, true);
    }
}
