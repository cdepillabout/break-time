//! Quartz (macOS) display backend.
//!
//! Idle detection is implemented (via `CGEventSource`), and
//! [`visible_app_names`] provides the owner-name window enumeration macOS
//! meeting detection uses. The *trait's* X11-shaped `list_windows`/`WindowInfo`
//! stays a stub by decision: reading other apps' window **titles** requires the
//! Screen Recording permission on macOS, so the macOS meeting plugins are built
//! on permission-free owner names + camera/mic-in-use instead (see
//! `scheduler::plugins`). Active-window save/restore is likewise unused here —
//! focus restore is done app-wise by the macOS break window through
//! `NSWorkspace` (see the note on the trait).

#![allow(unsafe_code)]

use std::time::Duration;

use objc2_core_foundation::{CFDictionary, CFString, CFType};
use objc2_core_graphics::{
    kCGNullWindowID, kCGWindowOwnerName, CGEventSource, CGEventSourceStateID,
    CGEventType, CGWindowListCopyWindowInfo, CGWindowListOption,
};

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
        // Deliberately empty (see the module docs): the X11-shaped WindowInfo
        // does not fit macOS, and the macOS meeting plugins use
        // `visible_app_names` + camera/mic-in-use instead. An empty list means
        // the (Linux) window-title plugin would always allow a break.
        Ok(Vec::new())
    }

    fn save_active_window(&self) -> Option<WindowRef> {
        None
    }

    fn restore_active_window(&self, _win: WindowRef) {}
}

/// The owner (application) names of all on-screen windows, deduplicated —
/// e.g. `["Google Chrome", "Terminal", "zoom.us"]`. Owner names are readable
/// without any permission (unlike window titles, which need Screen Recording).
/// `Err` means the window list itself could not be read.
pub fn visible_app_names() -> Result<Vec<String>, ()> {
    let list = CGWindowListCopyWindowInfo(
        CGWindowListOption::OptionOnScreenOnly
            | CGWindowListOption::ExcludeDesktopElements,
        kCGNullWindowID,
    )
    .ok_or(())?;

    let mut names = Vec::new();
    for i in 0..list.count() {
        // Each element of a CGWindowList is documented to be a CFDictionary
        // describing one window.
        let dict = unsafe {
            &*list
                .value_at_index(i)
                .cast::<CFDictionary<CFString, CFType>>()
        };
        if let Some(name) =
            dict.get(unsafe { kCGWindowOwnerName }).and_then(|owner| {
                owner.downcast_ref::<CFString>().map(CFString::to_string)
            })
        {
            names.push(name);
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}
