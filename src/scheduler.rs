
mod plugins;

use plugins::Plugin;

use std::time::Duration;
use std::sync::{Arc, Mutex};

use super::Msg;

pub struct Scheduler {
    sender: glib::Sender<Msg>,
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
    time_until_break: Duration,
}

const DEFAULT_TIME_UNTIL_BREAK: Duration = Duration::from_secs(5 * 60);

impl Scheduler {
    pub fn new(sender: glib::Sender<Msg>) -> Self {
        let window_title_plugin = plugins::WindowTitles::new();
        Scheduler {
            sender,
            plugins: vec![Box::new(window_title_plugin)],
            time_until_break: DEFAULT_TIME_UNTIL_BREAK,
        }
    }

    pub fn run(self) {
        std::thread::spawn(move || {
            std::thread::sleep(self.time_until_break);
        });
    }
}
