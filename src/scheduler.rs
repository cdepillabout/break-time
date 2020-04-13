mod plugins;

use plugins::{CanBreak, Plugin};

use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    Start,
}

pub struct Plugins(Vec<Box<dyn Plugin>>);

impl Plugins {
    fn new() -> Result<Self, ()> {
        let window_title_plugin = plugins::WindowTitles::new()?;
        let google_calendar_plugin = plugins::GoogleCalendar::new()?;
        let all_plugins: Vec<Box<dyn Plugin>> =
            vec![Box::new(window_title_plugin), Box::new(google_calendar_plugin)];
        Ok(Plugins(all_plugins))
    }

    fn can_break_now(&self) -> (Option<CanBreak>, Vec<()>) {
        fn f(
            (opt_old_can_break, mut err_accum): (Option<CanBreak>, Vec<()>),
            plugin: &Box<dyn Plugin>,
        ) -> (Option<CanBreak>, Vec<()>) {
            let res_can_break = plugin.can_break_now();
            match res_can_break {
                Err(err) => {
                    err_accum.push(err);
                    (opt_old_can_break, err_accum)
                }
                Ok(can_break) => {
                    let new_can_break = opt_old_can_break
                        .map_or(can_break, |old_can_break| {
                            can_break.combine(&old_can_break)
                        });
                    (Some(new_can_break), err_accum)
                }
            }
        }

        // TODO: I probably want to parallelize calling can_break_now()
        // for each of the plugins, because they may take a non-trivial
        // amount of time deciding whether or not to break.
        self.iter()
            .fold((None, vec![]), |accum, plugin| f(accum, plugin))
    }
}

impl std::ops::Deref for Plugins {
    type Target = [Box<dyn Plugin>];

    fn deref(&self) -> &[Box<dyn Plugin>] {
        &self.0
    }
}

pub struct Scheduler {
    sender: glib::Sender<super::Msg>,
    plugins: Plugins,
    time_until_break: Duration,
}

const DEFAULT_TIME_UNTIL_BREAK: Duration = Duration::from_secs(1 * 10);

impl Scheduler {
    pub fn new(sender: glib::Sender<super::Msg>) -> Result<Self, ()> {
        Ok(Scheduler {
            sender,
            plugins: Plugins::new()?,
            time_until_break: DEFAULT_TIME_UNTIL_BREAK,
        })
    }

    pub fn run(sender: glib::Sender<super::Msg>) -> Sender<Msg> {
        let (sched_sender, sched_receiver) = channel();
        std::thread::spawn(move || {
            // TODO: Need to actually handle this error.
            let sched = Scheduler::new(sender).expect("Could not initialize plugins.");
            loop {
                sched.wait_until_break();

                // The only kind of message we have so far is Start, which means we should just
                // continue running this loop.
                let _msg = sched_receiver.recv().expect("Error receiving value in Scheduler.");
            }
        });
        sched_sender
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
                            self.sender.send(super::Msg::StartBreak);
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
