//! OS-layer selection (compile time). Each OS module provides the concrete tray
//! type (re-exported as `TrayImpl`) and, in later steps, the break-window entry
//! point and the display-backend constructor.
//!
//! There is intentionally no `Os` trait or `OsImpl` struct yet: with a single OS
//! there is no polymorphism to justify one, so selection is expressed by
//! `#[cfg(target_os)]`-gated re-exports of concrete types/functions. A trait can
//! be introduced later if a second OS makes it pull its weight.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::tray::{IsIdleDetectorEnabled, Tray as TrayImpl};

// macOS arm (NSStatusItem tray, Cocoa break window, Quartz display backend) is
// added alongside the macOS sketches:
//   #[cfg(target_os = "macos")] pub mod macos;
//   #[cfg(target_os = "macos")] pub use macos::tray::{IsIdleDetectorEnabled, Tray as TrayImpl};
