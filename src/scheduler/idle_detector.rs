
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
use std::time::Duration;

use crate::config::Config;

pub struct IdleDetector {
    conn: xcb::Connection,
    root_window: xcb::Window,
    restart_wait_time_sender: Sender<()>,
}

impl IdleDetector {
    pub fn new(restart_wait_time_sender: Sender<()>) -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let setup: xcb::Setup = conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let screen: xcb::Screen = roots.nth(screen_num as usize).unwrap();
        let root_window: xcb::Window = screen.root();

        Self {
            conn,
            root_window,
            restart_wait_time_sender,
        }
    }

    pub fn run(config: &Config, restart_wait_time_sender: Sender<()>) -> ! {
        let idle_detector = Self::new(restart_wait_time_sender);
        let idle_detection_milliseconds = config.settings.idle_detection_seconds * 1000;
        loop {
            std::thread::sleep(Duration::from_secs(20));

            let idle_query_res = xcb::screensaver::query_info(&idle_detector.conn, idle_detector.root_window)
                .get_reply()
                .unwrap();

            let ms_since_user_input = idle_query_res.ms_since_user_input();

            println!(
                "state: {}, ms_since_user_input: {}, kind: {}",
                idle_query_res.state(),
                ms_since_user_input,
                idle_query_res.kind()
            );

            if ms_since_user_input >= idle_detection_milliseconds {
                idle_detector.restart_wait_time_sender.send(());
            }
        }
    }
}
