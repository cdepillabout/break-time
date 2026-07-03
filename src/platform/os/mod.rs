//! OS-layer selection (compile time). Each OS module provides the concrete tray
//! type (re-exported as `TrayImpl`), the break-window entry point
//! (`start_break`), the display-backend constructor (`create_display`), and the
//! app runtime (`app`: the main loop + `Msg` channel).
//!
//! There is intentionally no `Os` trait or `OsImpl` struct: with the OS chosen
//! at compile time there is no polymorphism to justify one, so selection is
//! expressed by `#[cfg(target_os)]`-gated re-exports of concrete types and
//! functions.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub use linux::{
    app::{channel, quit, run_main_loop, AppSender},
    create_display, init_logging, start_break,
    tray::{IsIdleDetectorEnabled, Tray as TrayImpl},
};

#[cfg(target_os = "macos")]
pub use macos::{
    app::{channel, quit, run_main_loop, AppSender},
    create_display, init_logging, start_break,
    tray::{IsIdleDetectorEnabled, Tray as TrayImpl},
};
