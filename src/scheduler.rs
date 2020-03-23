
mod plugins;

use plugins::{CanBreak, Plugin};

use std::time::Duration;
use std::sync::{Arc, Mutex};

use super::Msg;

#[derive(Clone)]
pub struct Plugins(Arc<Vec<Box<dyn Plugin + Send + Sync>>>);

impl Plugins {
    fn new() -> Result<Self, ()> {
        let window_title_plugin = plugins::WindowTitles::new()?;
        let all_plugins: Vec<Box<dyn Plugin + Send + Sync>> = vec![Box::new(window_title_plugin)];
        Ok(Plugins(Arc::new(all_plugins)))
    }

    fn can_break_now(&self) -> (Option<CanBreak>, Vec<()>) {
        self.iter().fold((None, vec![]), |(opt_old_can_break, mut err_accum): (Option<CanBreak>, Vec<()>), plugin: &Box<dyn Plugin + Send + Sync>| {
            let res_can_break = plugin.can_break_now();
            match res_can_break {
                Err(err) => {
                    err_accum.push(err);
                    (opt_old_can_break, err_accum)
                }
                Ok(can_break) => {
                    let new_can_break = opt_old_can_break.map_or(can_break, |old_can_break| can_break.combine(&old_can_break));
                    (Some(new_can_break), err_accum)
                }
            }
        })
    }

}

impl std::ops::Deref for Plugins {
    type Target = [Box<dyn Plugin + Send + Sync>];

    fn deref(&self) -> &[Box<dyn Plugin + Send + Sync>] {
        &self.0
    }
}

#[derive(Clone)]
pub struct Scheduler {
    sender: glib::Sender<Msg>,
    plugins: Plugins,
    time_until_break: Duration,
}

const DEFAULT_TIME_UNTIL_BREAK: Duration = Duration::from_secs(1 * 10);

impl Scheduler {
    pub fn new(sender: glib::Sender<Msg>) -> Result<Self, ()> {
        Ok(Scheduler {
            sender,
            plugins: Plugins::new()?,
            time_until_break: DEFAULT_TIME_UNTIL_BREAK,
        })
    }

    pub fn run(&self) {
        let sched = self.clone();
        std::thread::spawn(move || sched.wait_until_break());
    }

    pub fn wait_until_break(&self) {
        loop {
            std::thread::sleep(self.time_until_break);
            let (opt_can_break, errs) = self.plugins.can_break_now();
            if errs.is_empty() {
                match opt_can_break {
                    None => panic!("If there are no errors, then we should always get a response to can_break"),
                    Some(can_break) => {
                        if can_break.into_bool() {
                            self.sender.send(Msg::StartBreak);
                            break;
                        } else {
                            println!("Could not break right now, so sleeping again...");
                        }
                    }
                }
            } else {
                println!("There have been some errors from our plugins, sleeping again just to be safe...");
            }
        }
    }
}
