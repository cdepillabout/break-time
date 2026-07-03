//! macOS OS-layer implementation: the `NSApplication` run loop and app message
//! channel (`app`), the native Cocoa break window (`break_window`), the system
//! tray (`tray`, an `NSStatusItem` — stubbed for now), and the Quartz display
//! backend.

pub mod app;
pub mod break_window;
pub mod logging;
pub mod media;
pub mod tray;

pub use break_window::start_break;
pub use logging::init as init_logging;

use crate::platform::display::DisplayBackend;

/// Build the display backend for this session. macOS has exactly one option, the
/// Quartz backend.
#[must_use]
pub fn create_display() -> Box<dyn DisplayBackend> {
    Box::new(crate::platform::display::quartz::QuartzBackend::new())
}
