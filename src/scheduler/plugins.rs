mod window_titles;

pub use window_titles::WindowTitles;

pub trait Plugin {
    fn can_break_now(&self) -> Result<bool, ()>;
}
