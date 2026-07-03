//! Linux OS-layer implementation: the `GtkStatusIcon` system tray, the GTK
//! break-window GUI, and the display-backend constructor.

pub mod app;
pub mod break_window;
pub mod tray;

pub use break_window::start_break;

use crate::platform::display::DisplayBackend;

/// No-op on Linux: diagnostic output goes to stdout.
pub fn init_logging() {}

/// Build the display backend for this session. Cargo features decide which
/// backends are compiled in; when more than one is present, runtime detection
/// picks among them. Today only X11 is implemented (it also covers Wayland
/// sessions via `XWayland`).
pub fn create_display() -> Box<dyn DisplayBackend> {
    // A `#[cfg(feature = "wayland")]` arm that picks Wayland vs X11 at runtime
    // slots in here once a Wayland backend exists.
    #[cfg(feature = "x11")]
    {
        Box::new(crate::platform::display::x11::X11Backend::connect())
    }
    #[cfg(not(feature = "x11"))]
    compile_error!("enable at least one display backend (the `x11` feature)");
}
