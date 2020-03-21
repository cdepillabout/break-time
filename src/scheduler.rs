
mod plugins;

use plugins::Plugin;

use std::time::Duration;
use std::sync::{Arc, Mutex};

use super::Msg;

#[derive(Clone)]
pub struct Scheduler {
    sender: glib::Sender<Msg>,
    plugins: Arc<Vec<Box<dyn Plugin + Send + Sync>>>,
    time_until_break: Duration,
}

const DEFAULT_TIME_UNTIL_BREAK: Duration = Duration::from_secs(1 * 10);

impl Scheduler {
    pub fn new(sender: glib::Sender<Msg>) -> Self {
        let window_title_plugin = plugins::WindowTitles::new();
        Scheduler {
            sender,
            plugins: Arc::new(vec![Box::new(window_title_plugin)]),
            time_until_break: DEFAULT_TIME_UNTIL_BREAK,
        }
    }

    pub fn run(&self) {
        let time_until_break = self.time_until_break;
        let sender = self.sender.clone();
        let sched = self.clone();
        std::thread::spawn(move || sched.wait_until_break());
    }

    pub fn wait_until_break(&self) {
        std::thread::sleep(self.time_until_break);
        self.sender.send(Msg::StartBreak);
    }
}
