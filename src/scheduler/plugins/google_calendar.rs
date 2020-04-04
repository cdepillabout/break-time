use super::{CanBreak, Plugin};

pub struct GoogleCalendar {
}

impl GoogleCalendar {
    pub fn new() -> Result<Self, ()> {
        Ok(GoogleCalendar {
        })
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        Ok(CanBreak::Yes)
    }
}

impl Plugin for GoogleCalendar {
    fn can_break_now(&self) -> Result<CanBreak, ()> {
        self.can_break()
    }
}
