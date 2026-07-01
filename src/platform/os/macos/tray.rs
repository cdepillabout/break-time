//! macOS system tray.
//!
//! STUB for the initial MVP — a real `NSStatusItem` implementation (with the
//! per-second countdown redraw the Linux `GtkStatusIcon` does) lands in a later
//! commit. For now every method is a no-op so the cross-platform `Msg` dispatch
//! in `lib.rs` compiles and runs unchanged; the app simply has no menu-bar
//! presence yet.
//!
//! These lint allows are because the methods are deliberately empty no-ops for
//! now; they gain real bodies (drawing onto the `NSStatusItem`, using `self` and
//! `&mut self`) when the real tray lands.
#![allow(
    clippy::unused_self,
    clippy::needless_pass_by_ref_mut,
    clippy::missing_const_for_fn
)]

use std::time::Duration;

use crate::config::Config;
use crate::platform::AppSender;

#[derive(Copy, Clone, Debug)]
pub enum IsIdleDetectorEnabled {
    Yes,
    No,
}

pub struct Tray;

impl Tray {
    #[must_use]
    pub fn run(_config: &Config, _sender: AppSender) -> Self {
        Self
    }

    pub fn render_break_starting(&self) {}

    pub fn render_normal_icon(&self) {}

    pub fn break_end(&self) {}

    pub fn pause(&mut self) {}

    pub fn resume(&mut self) {}

    pub fn update_time_remaining(&self, _remaining_time: Duration) {}

    pub fn set_is_idle_detector_enabled(
        &mut self,
        _is_idle_detector_enabled: IsIdleDetectorEnabled,
    ) {
    }
}
