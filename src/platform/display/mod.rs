//! Display-server abstraction (the axis that owns input and windows): idle
//! detection, active-window save/restore, and window enumeration. Backends are
//! compiled in by cargo feature (`x11`, later `wayland`) and, when more than one
//! is present, chosen at runtime. X11 today; Wayland/Quartz later.
//!
//! The trait grows one method per migrated consumer rather than landing
//! fully-formed, so there is never an unused/stub method.
//!
//! Note: the input grab (the lock) is intentionally NOT on this trait. On Linux
//! it is a GDK seat grab tied to the GTK break window, and on macOS it is a
//! `CGEventTap` tied to the native break windows — i.e. it is coupled to the
//! break-window presentation on both platforms, so it lives in the OS layer.

#[cfg(feature = "x11")]
pub mod x11;

use std::time::Duration;

/// An opaque handle to a display-server window, used to remember the active
/// window at break start and restore focus to it at break end. On X11 this
/// wraps an `xproto::Window`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WindowRef(pub u32);

/// One window's properties, as needed by the cross-platform meeting-detection
/// predicates. Each field is independently fallible: on X11 each comes from a
/// separate property request that can fail or be absent. The `Err` case is kept
/// (rather than flattened to an empty string) because the predicates treat a
/// failed read as "can break".
#[derive(Clone, Debug)]
pub struct WindowInfo {
    #[allow(dead_code)]
    pub wm_name: Result<String, ()>,
    pub net_wm_name: Result<String, ()>,
    pub class_name: Result<String, ()>,
    pub class: Result<String, ()>,
    #[allow(dead_code)]
    pub transient_for: Result<Vec<WindowRef>, ()>,
}

/// A connection to the display server that owns input and windows.
pub trait DisplayBackend {
    /// Time since the last user input (keyboard or pointer).
    fn idle_time(&self) -> Duration;

    /// All top-level windows with the properties meeting detection needs. `Err`
    /// signals that enumeration failed (distinct from "no windows"); the caller
    /// treats that as "do not break right now".
    fn list_windows(&self) -> Result<Vec<WindowInfo>, ()>;

    /// The currently-active window, so it can be re-focused when the break ends.
    /// `None` if there is no active window or it cannot be determined.
    fn save_active_window(&self) -> Option<WindowRef>;

    /// Re-focus a window previously returned by
    /// [`save_active_window`](Self::save_active_window). Best-effort: failures
    /// are logged, not returned.
    fn restore_active_window(&self, win: WindowRef);
}
