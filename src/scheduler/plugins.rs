mod window_titles;

pub use window_titles::WindowTitles;

enum CanBreak {
    Yes,
    No,
}

impl CanBreak {
    fn into_bool(self) -> bool {
        match self {
            CanBreak::Yes => true,
            CanBreak::No => false,
        }
    }

    fn from_bool(b: bool) -> Self {
        match b {
            true => CanBreak::Yes,
            false => CanBreak::No,
        }
    }
}

pub trait Plugin {
    fn can_break_now(&self) -> Result<CanBreak, ()>;
}
