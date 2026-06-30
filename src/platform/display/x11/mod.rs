//! X11 display backend, built on the pure-Rust `x11rb` protocol implementation.
//! Wraps the low-level [`X11`] connection helper.

use std::time::Duration;

use x11rb::protocol::screensaver::ConnectionExt as _;
use x11rb::protocol::xproto::Window;

use super::DisplayBackend;
use crate::x11::X11;

pub struct X11Backend {
    x11: X11,
    root: Window,
}

impl X11Backend {
    pub fn connect() -> Self {
        let x11 = X11::connect();
        let root = x11
            .get_root_win()
            .expect("X11: could not determine the root window");
        Self { x11, root }
    }
}

impl DisplayBackend for X11Backend {
    fn idle_time(&self) -> Duration {
        // Matches the previous behaviour of the idle detector: a failed query
        // panics (rather than silently reporting zero idle time, which would
        // suppress idle detection).
        let info = self
            .x11
            .conn
            .screensaver_query_info(self.root)
            .expect("X11: ScreenSaverQueryInfo request failed")
            .reply()
            .expect("X11: ScreenSaverQueryInfo reply failed");
        Duration::from_millis(u64::from(info.ms_since_user_input))
    }
}
