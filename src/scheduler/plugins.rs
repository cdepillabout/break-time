#[cfg(target_os = "macos")]
mod camera_mic;
#[cfg(feature = "google-calendar")]
pub mod google_calendar;
#[cfg(target_os = "macos")]
mod meeting_apps;
// Window-title matching is X11-shaped end to end (WM_CLASS values, X11 browser
// title heuristics), so it is a Linux-only plugin; macOS uses the camera/mic
// and meeting-apps plugins instead.
#[cfg(target_os = "linux")]
mod window_titles;

#[cfg(target_os = "macos")]
pub use camera_mic::CameraMic;
#[cfg(feature = "google-calendar")]
pub use google_calendar::GoogleCalendar;
#[cfg(target_os = "macos")]
pub use meeting_apps::MeetingApps;
#[cfg(target_os = "linux")]
pub use window_titles::WindowTitles;

#[derive(Copy, Clone, Debug)]
pub enum CanBreak {
    Yes,
    No,
}

impl CanBreak {
    pub const fn into_bool(self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
        }
    }

    pub const fn from_bool(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }

    pub const fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::No, _) | (_, Self::No) => Self::No,
            _ => Self::Yes,
        }
    }
}

pub trait Plugin {
    /// A short stable name for log lines (which plugin allowed/vetoed a break).
    fn name(&self) -> &'static str;

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>>;
}

impl Plugin for Box<dyn Plugin> {
    fn name(&self) -> &'static str {
        (**self).name()
    }

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        (**self).can_break_now()
    }
}
