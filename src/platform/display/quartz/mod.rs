//! Quartz (macOS) display backend.
//!
//! STUBS for the initial MVP: idle detection, window enumeration, and
//! active-window save/restore land in later commits (via
//! `CGEventSourceSecondsSinceLastEventType`, `CGWindowListCopyWindowInfo`, and
//! `NSWorkspace`). The neutral values returned here keep the scheduler's idle
//! detector and the window-title plugin inert — never resetting or vetoing a
//! break — rather than misbehaving.

use std::time::Duration;

use crate::platform::display::{DisplayBackend, WindowInfo, WindowRef};

pub struct QuartzBackend;

impl QuartzBackend {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for QuartzBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayBackend for QuartzBackend {
    fn idle_time(&self) -> Duration {
        // No idle detection yet: report "never idle" so the idle detector never
        // resets the countdown.
        Duration::ZERO
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>, ()> {
        // No window enumeration yet: an empty list means the window-title plugin
        // sees no meeting windows and always allows a break.
        Ok(Vec::new())
    }

    fn save_active_window(&self) -> Option<WindowRef> {
        None
    }

    fn restore_active_window(&self, _win: WindowRef) {}
}
