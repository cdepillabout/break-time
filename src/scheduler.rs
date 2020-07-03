mod plugins;

use plugins::{CanBreak, Plugin};
use super::config::Config;
use super::tray::Tray;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    Start,
}

pub struct Plugins(Vec<Box<dyn Plugin>>);

impl Plugins {
    fn new(config: &Config) -> Result<Self, ()> {
        let window_title_plugin = plugins::WindowTitles::new(config)?;
        let google_calendar_plugin = plugins::GoogleCalendar::new(config)?;
        let all_plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(window_title_plugin),
            Box::new(google_calendar_plugin),
        ];
        Ok(Plugins(all_plugins))
    }

    fn can_break_now(&self) -> (Option<CanBreak>, Vec<Box<dyn std::error::Error>>) {
        fn f(
            (opt_old_can_break, mut err_accum): (Option<CanBreak>, Vec<Box<dyn std::error::Error>>),
            plugin: &Box<dyn Plugin>,
        ) -> (Option<CanBreak>, Vec<Box<dyn std::error::Error>>) {
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

enum State {
    CountDownToBreak,
    WaitingForBreakEnd,
}

pub struct Scheduler {
    sender: glib::Sender<super::Msg>,
    plugins: Plugins,
    time_until_break: Duration,
    receiver: Receiver<Msg>,
    state: State,
}

impl Scheduler {
    pub fn new(config: &Config, sender: glib::Sender<super::Msg>, receiver: Receiver<Msg>) -> Result<Self, ()> {
        Ok(Scheduler {
            sender,
            plugins: Plugins::new(config)?,
            time_until_break: Duration::from_secs(config.settings.seconds_between_breaks.into()),
            receiver,
            state: State::CountDownToBreak,
        })
    }

    pub fn run(config: &Config, sender: glib::Sender<super::Msg>) -> Sender<Msg> {
        let (sched_sender, sched_receiver) = channel();
        let config_clone = config.clone();
        std::thread::spawn(move || {
            // TODO: Need to actually handle this error.
            let mut sched =
                Scheduler::new(&config_clone, sender, sched_receiver).expect("Could not initialize plugins.");
            println!("Scheduler initialized plugins");
            sched.run_loop();
        });
        sched_sender
    }

    fn run_loop(&mut self) -> ! {
      loop {
          match self.state {
              State::CountDownToBreak => {
                  self.wait_until_break();
                  self.state = State::WaitingForBreakEnd;
              }
              State::WaitingForBreakEnd => {
                  // Wait for a message signalling a break ending.
                  let msg = self.receiver
                      .recv()
                      .expect("Error receiving value in Scheduler.");

                  match msg {
                      Msg::Start => {
                          self.state = State::CountDownToBreak;
                      }
                  }
              }
          }
      }
    }

    fn wait_until_break(&self) {
        loop {
            self.blahblah();
            println!(
                "Scheduler finished sleeping, checking if it can break now..."
            );
            let (opt_can_break, errs) = self.plugins.can_break_now();
            if errs.is_empty() {
                match opt_can_break {
                    None => panic!("If there are no errors, then we should always get a response to can_break"),
                    Some(can_break) => {
                        if can_break.into_bool() {
                            println!("Scheduler realized it was able to break, so sending a message.");
                            self.sender.send(super::Msg::StartBreak);
                            break;
                        } else {
                            println!("Could not break right now, so sleeping again...");
                        }
                    }
                }
            } else {
                println!("There have been some errors from our plugins:");
                for e in errs {
                    println!("{}", e);
                }
                println!("Sleeping again just to be safe...");
            }
        }
    }

    fn blahblah(&self) {
        let mut remaining_time = self.time_until_break;
        println!("Starting blahblahblah...");
        for &period in PERIODS_TO_SEND_TIME_LEFT_MESSAGE.iter() {
            let opt_time_to_sleep = remaining_time.checked_sub(period);
            println!("================================================================");
            println!("In blahblahblah loop for period {:?}, remaining_time: {:?}, time_to_sleep: {:?}", period, remaining_time, opt_time_to_sleep);
            match opt_time_to_sleep {
                None => {
                }
                Some(time_to_sleep) => {
                    std::thread::sleep(time_to_sleep);
                    self.sender.send(super::Msg::TimeRemainingBeforeBreak(period));
                    remaining_time -= time_to_sleep;
                }
            }
        }
    }
}

const PERIODS_TO_SEND_TIME_LEFT_MESSAGE: [Duration; 22] = [
    Duration::from_secs(60 * 5),
    Duration::from_secs(60 * 4),
    Duration::from_secs(60 * 3),
    Duration::from_secs(60 * 2),
    Duration::from_secs(60 * 1),
    Duration::from_secs(30),
    Duration::from_secs(15),
    Duration::from_secs(14),
    Duration::from_secs(13),
    Duration::from_secs(12),
    Duration::from_secs(11),
    Duration::from_secs(10),
    Duration::from_secs(9),
    Duration::from_secs(8),
    Duration::from_secs(7),
    Duration::from_secs(6),
    Duration::from_secs(5),
    Duration::from_secs(4),
    Duration::from_secs(3),
    Duration::from_secs(2),
    Duration::from_secs(1),
    Duration::from_secs(0),
    ];
