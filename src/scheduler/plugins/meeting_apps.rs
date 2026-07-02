//! macOS meeting detection, secondary signal: does a known meeting app have a
//! window on screen? The analog of the Linux window-title plugin's coarse
//! native-app checks (any Zoom/Skype window vetoes a break, in a meeting or
//! not). Matches on window *owner names* from `CGWindowList`, which are
//! readable without any permission — window titles are not (Screen Recording),
//! which is why the browser-tab checks have no analog here (the camera/mic
//! plugin covers those).
//!
//! Note (verified): a meeting app that is merely *running* — e.g. Zoom's
//! menu-bar item with no open windows — does NOT veto, because only on-screen
//! windows are enumerated. That is intended: an idle background Zoom should
//! not block breaks, and an actual meeting is caught by the camera/mic plugin
//! regardless of windows.

use super::{CanBreak, Plugin};

use crate::config::Config;
use crate::platform::display::quartz;

/// Window owner names that veto a break while on screen. The native apps the
/// Linux plugin also vetoes; a config-driven list is a Phase 4 backlog item.
/// Zoom's owner name varies by client generation — the current Zoom
/// (Workplace) client reports "Zoom" (verified locally); older clients report
/// "zoom.us" — so both are listed.
const MEETING_APP_NAMES: &[&str] =
    &["Zoom", "Zoom Workplace", "zoom.us", "Skype"];

pub struct MeetingApps;

impl MeetingApps {
    #[must_use]
    pub const fn new(_config: &Config) -> Self {
        Self
    }
}

impl Plugin for MeetingApps {
    fn name(&self) -> &'static str {
        "meeting_apps"
    }

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        let names = quartz::visible_app_names().map_err(|()| {
            Box::new(std::io::Error::other(
                "could not enumerate on-screen windows (CGWindowList)",
            )) as Box<dyn std::error::Error>
        })?;
        let meeting_app_on_screen = names
            .iter()
            .any(|name| MEETING_APP_NAMES.contains(&name.as_str()));
        Ok(CanBreak::from_bool(!meeting_app_on_screen))
    }
}
