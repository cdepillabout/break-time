//! Quartz (macOS) display backend.
//!
//! Idle detection is implemented (via `CGEventSource`). Window enumeration and
//! active-window save/restore are still STUBS: enumeration lands with macOS
//! meeting detection (`CGWindowListCopyWindowInfo`), and focus restore is done
//! app-wise by the macOS break window through `NSWorkspace` instead (see the
//! note on the trait). The neutral stub values keep the window-title plugin
//! inert — never vetoing a break — rather than misbehaving.

use std::time::Duration;

use objc2_core_graphics::{CGEventSource, CGEventSourceStateID, CGEventType};

use crate::platform::display::{DisplayBackend, WindowInfo, WindowRef};

/// `kCGAnyInputEventType` — "time since *any* HID input" (keyboard, mouse
/// movement, clicks, scroll). Defined as `~0` in the CoreGraphics headers but
/// not exposed as a named constant by `objc2-core-graphics`.
const ANY_INPUT_EVENT_TYPE: CGEventType = CGEventType(u32::MAX);

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
        // Seconds since the last user input (keyboard or pointer) — the direct
        // macOS analog of the X11 screensaver query, and the same call
        // Chromium/Electron (and hence Stretchly/BreakTimer) use for idle
        // time. `CombinedSessionState` scopes the counter to the current login
        // session, so under fast user switching another user's input does not
        // count as ours (unlike the hardware-wide `HIDSystemState`).
        let secs = CGEventSource::seconds_since_last_event_type(
            CGEventSourceStateID::CombinedSessionState,
            ANY_INPUT_EVENT_TYPE,
        );
        // `try_from_secs_f64` fails on negative/NaN/overflow, none of which the
        // counter should ever produce; treat them as "not idle" rather than
        // taking down the idle-detector thread.
        Duration::try_from_secs_f64(secs).unwrap_or(Duration::ZERO)
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
