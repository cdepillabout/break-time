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

/// A connection to the display server that owns input and windows.
pub trait DisplayBackend {
    /// Time since the last user input (keyboard or pointer).
    fn idle_time(&self) -> Duration;
}
