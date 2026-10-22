//! macOS meeting detection: any camera activity or microphone activity from
//! a process outside the built-in ignore list postpones breaks. This covers
//! native meeting apps and browser calls without Screen Recording permission.
//! See `platform::os::macos::media` for the queries.

use super::{CanBreak, Plugin};

use crate::config::Config;
use crate::platform::os::macos::media;

pub struct CameraMic;

impl CameraMic {
    #[must_use]
    pub const fn new(_config: &Config) -> Self {
        Self
    }
}

impl Plugin for CameraMic {
    fn name(&self) -> &'static str {
        "camera_mic"
    }

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        let camera = media::camera_running().map_err(|()| {
            Box::new(std::io::Error::other(
                "could not enumerate cameras (CoreMediaIO)",
            )) as Box<dyn std::error::Error>
        })?;
        let mic = media::mic_running().map_err(|()| {
            Box::new(std::io::Error::other(
                "could not enumerate audio input devices (CoreAudio)",
            )) as Box<dyn std::error::Error>
        })?;
        let can_break = !camera && !mic;
        println!("camera_mic: decision camera_active={camera} microphone_blocks={mic} can_break={can_break}");
        Ok(CanBreak::from_bool(can_break))
    }
}
