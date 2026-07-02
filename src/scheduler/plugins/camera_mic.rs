//! macOS meeting detection, primary signal: is the camera or the microphone in
//! use by any process? Covers native meeting apps *actually in a call* and any
//! browser-tab meeting (WebRTC always opens the microphone) — the cases the
//! Linux window-title plugin matches by title, which is not possible
//! permission-free on macOS. See `platform::os::macos::media` for the queries.

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
        Ok(CanBreak::from_bool(!camera && !mic))
    }
}
