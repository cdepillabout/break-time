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
            CanBreak::Yes => true,
            CanBreak::No => false,
        }
    }

    pub fn from_bool(b: bool) -> Self {
        match b {
            true => CanBreak::Yes,
            false => CanBreak::No,
        }
    }

    pub fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (CanBreak::No, _) => CanBreak::No,
            (_, CanBreak::No) => CanBreak::No,
            _ => CanBreak::Yes,
        }
    }
}

pub trait Plugin {
    fn can_break_now(&self) -> Result<CanBreak, ()>;
}
