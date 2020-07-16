// This code is pretty horrible.  I am sorry.

mod idle_detector;
mod plugins;

use super::config::Config;
use super::tray::Tray;
use idle_detector::IdleDetector;
use plugins::{CanBreak, Plugin};

use std::sync::mpsc::{channel, Receiver, Sender};
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

    fn can_break_now(
        &self,
    ) -> (Option<CanBreak>, Vec<Box<dyn std::error::Error>>) {
        fn f(
            (opt_old_can_break, mut err_accum): (
                Option<CanBreak>,
                Vec<Box<dyn std::error::Error>>,
            ),
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
    Paused,
    WaitingForBreakEnd,
}

pub struct Scheduler {
    sender: glib::Sender<super::Msg>,
    plugins: Plugins,
    time_until_break: Duration,
    break_ending_receiver: Receiver<Msg>,
    restart_wait_time_receiver: Receiver<InnerMsg>,
    state: State,
}

enum WaitUntilBreakResult {
    FinishedWaiting,
    Paused,
}

impl Scheduler {
    pub fn new(
        config: &Config,
        sender: glib::Sender<super::Msg>,
        break_ending_receiver: Receiver<Msg>,
        restart_wait_time_receiver: Receiver<InnerMsg>,
    ) -> Result<Self, ()> {
        Ok(Scheduler {
            sender,
            plugins: Plugins::new(config)?,
            time_until_break: Duration::from_secs(
                config.settings.seconds_between_breaks.into(),
            ),
            break_ending_receiver,
            restart_wait_time_receiver,
            state: State::CountDownToBreak,
        })
    }

    pub fn run(
        config: &Config,
        sender: glib::Sender<super::Msg>,
    ) -> (Sender<Msg>, Sender<InnerMsg>) {
        let (sched_break_ending_sender, sched_break_ending_receiver) =
            channel();
        let (restart_wait_time_sender, restart_wait_time_receiver) = channel();
        let config_clone = config.clone();
        std::thread::spawn(move || {
            // TODO: Need to actually handle this error.
            let mut sched = Scheduler::new(
                &config_clone,
                sender,
                sched_break_ending_receiver,
                restart_wait_time_receiver,
            )
            .expect("Could not initialize plugins.");
            println!("Scheduler initialized plugins");
            sched.run_loop();
        });
        let config_clone = config.clone();
        let restart_wait_time_sender_clone = restart_wait_time_sender.clone();
        std::thread::spawn(move || {
            IdleDetector::run(&config_clone, restart_wait_time_sender_clone);
        });
        (sched_break_ending_sender, restart_wait_time_sender)
    }

    fn run_loop(&mut self) -> ! {
        loop {
            match self.state {
                State::CountDownToBreak => {
                    let wait_until_break_result = self.wait_until_break();
                    match wait_until_break_result {
                        WaitUntilBreakResult::FinishedWaiting => {
                            self.state = State::WaitingForBreakEnd;
                        }
                        WaitUntilBreakResult::Paused => {
                            self.state = State::Paused;
                        }
                    }
                }
                State::Paused | State::WaitingForBreakEnd => {
                    // Wait for a message signalling a break ending or a pause ending.
                    println!("Scheduler currently waiting for a message signaling either a break or a pause ending.");
                    let msg = self
                        .break_ending_receiver
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

    fn wait_until_break(&self) -> WaitUntilBreakResult {
        loop {
            let waiting_result = self.send_msgs_while_waiting();
            match waiting_result {
                WaitingResult::Finished => {
                    println!(
                        "Scheduler successfully finished sleeping, checking if it can break now..."
                    );
                    let (opt_can_break, errs) = self.plugins.can_break_now();
                    if errs.is_empty() {
                        match opt_can_break {
                            None => panic!("If there are no errors, then we should always get a response to can_break"),
                            Some(can_break) => {
                                if can_break.into_bool() {
                                    println!("Scheduler realized it was able to break, so sending a message.");
                                    self.sender.send(super::Msg::StartBreak);
                                    return WaitUntilBreakResult::FinishedWaiting;
                                } else {
                                    println!("Could not break right now, so sleeping again...");
                                }
                            }
                        }
                    } else {
                        println!(
                            "There have been some errors from our plugins:"
                        );
                        for e in errs {
                            println!("{}", e);
                        }
                        println!("Sleeping again just to be safe...");
                    }
                }
                WaitingResult::NeedToRestart => {
                    // Just let this loop restart.
                    println!(
                        "Scheduler got a message to restart sleeping again, probably because X has been idle..."
                    );
                }
                WaitingResult::Paused => {
                    return WaitUntilBreakResult::Paused;
                }
            }
        }
    }

    fn send_msgs_while_waiting(&self) -> WaitingResult {
        self.sender.send(super::Msg::ResetSysTrayIcon);
        let mut remaining_time = self.time_until_break;
        for period in create_periods_to_send_time_left_message(self.time_until_break) {
            let opt_time_to_sleep = remaining_time.checked_sub(period);
            println!("In send_msgs_while_waiting loop for period {:?}, remaining_time: {:?}, time_to_sleep: {:?}", period, remaining_time, opt_time_to_sleep);
            match opt_time_to_sleep {
                None => {
                    // This happens when the periods to send the time-left message are greater than
                    // the remaining time.  We can just skip this.
                }
                Some(time_to_sleep) => {
                    let res = self
                        .restart_wait_time_receiver
                        .recv_timeout(time_to_sleep);
                    match res {
                        Ok(InnerMsg::HasBeenIdle) => {
                            println!("HERERERERE");
                            return WaitingResult::NeedToRestart;
                        }
                        Ok(InnerMsg::Pause) => {
                            println!("Got inner message to pause...");
                            return WaitingResult::Paused;
                        }
                        Err(_) => {
                            self.sender.send(
                                super::Msg::TimeRemainingBeforeBreak(period),
                            );
                            remaining_time -= time_to_sleep;
                        }
                    }
                }
            }
        }

        return WaitingResult::Finished;
    }
}

enum WaitingResult {
    Finished,
    NeedToRestart,
    Paused,
}

pub enum InnerMsg {
    Pause,
    HasBeenIdle,
}

fn create_periods_to_send_time_left_message(time_between_breaks: Duration) -> Vec<Duration> {
    let mut periods = vec![];
    let secs_between_breaks = time_between_breaks.as_secs();
    let mins_between_breaks = secs_between_breaks / 60;

    for min in (1..=mins_between_breaks).rev() {
        periods.push(Duration::from_secs(min * 60));
    }

    for sec in (0..=59).rev() {
        periods.push(Duration::from_secs(sec));
    }

    periods
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_periods_to_send_time_left_message() {
        let res = create_periods_to_send_time_left_message(Duration::from_secs(5 * 60));

        let periods_to_send_time_left_message_expected = vec![
            Duration::from_secs(60 * 5),
            Duration::from_secs(60 * 4),
            Duration::from_secs(60 * 3),
            Duration::from_secs(60 * 2),
            Duration::from_secs(60),
            Duration::from_secs(59),
            Duration::from_secs(58),
            Duration::from_secs(57),
            Duration::from_secs(56),
            Duration::from_secs(55),
            Duration::from_secs(54),
            Duration::from_secs(53),
            Duration::from_secs(52),
            Duration::from_secs(51),
            Duration::from_secs(50),
            Duration::from_secs(49),
            Duration::from_secs(48),
            Duration::from_secs(47),
            Duration::from_secs(46),
            Duration::from_secs(45),
            Duration::from_secs(44),
            Duration::from_secs(43),
            Duration::from_secs(42),
            Duration::from_secs(41),
            Duration::from_secs(40),
            Duration::from_secs(39),
            Duration::from_secs(38),
            Duration::from_secs(37),
            Duration::from_secs(36),
            Duration::from_secs(35),
            Duration::from_secs(34),
            Duration::from_secs(33),
            Duration::from_secs(32),
            Duration::from_secs(31),
            Duration::from_secs(30),
            Duration::from_secs(29),
            Duration::from_secs(28),
            Duration::from_secs(27),
            Duration::from_secs(26),
            Duration::from_secs(25),
            Duration::from_secs(24),
            Duration::from_secs(23),
            Duration::from_secs(22),
            Duration::from_secs(21),
            Duration::from_secs(20),
            Duration::from_secs(19),
            Duration::from_secs(18),
            Duration::from_secs(17),
            Duration::from_secs(16),
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

        assert_eq!(res, periods_to_send_time_left_message_expected);
    }
}
