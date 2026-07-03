//! Platform abstraction, split along two independent axes:
//!
//! - **OS axis** (`os`) — selected at compile time with `#[cfg(target_os = ...)]`:
//!   the system tray, the break-window GUI, and config/cache dirs. Linux today;
//!   macOS later.
//! - **Display-server axis** (`display`) — cargo features decide which backends
//!   compile in, runtime picks among them: idle detection, active-window
//!   save/restore, and window enumeration. X11 today; Wayland/Quartz later.
//!   (Added in a later step.)

pub mod display;
pub mod os;

pub use os::{
    channel, create_display, init_logging, quit, run_main_loop, start_break,
    AppSender, IsIdleDetectorEnabled, TrayImpl,
};
