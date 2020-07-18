mod google_calendar;
mod window_titles;

pub use google_calendar::GoogleCalendar;
pub use window_titles::WindowTitles;

#[derive(Copy, Clone, Debug)]
pub enum CanBreak {
    Yes,
    No,
}

impl CanBreak {
    pub fn into_bool(self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
        }
    }

    pub fn from_bool(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }

    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::No, _) => Self::No,
            (_, Self::No) => Self::No,
            _ => Self::Yes,
        }
    }
}

pub trait Plugin {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>>;
}

impl Plugin for Box<dyn Plugin> {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        (**self).can_break_now()
    }
}
