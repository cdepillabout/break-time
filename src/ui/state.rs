use super::builder;
use super::prelude::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::SystemTime;

use crate::config::Config;
use crate::Msg;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    End,
}

#[derive(Clone, Debug)]
pub struct Monitor {
    pub id: i32,
    pub monitor: gdk::Monitor,
}

impl Monitor {
    pub const fn new(id: i32, monitor: gdk::Monitor) -> Self {
        Self { id, monitor }
    }

    pub fn new_from_id(display: &gdk::Display, id: i32) -> Self {
        let mon = display.get_monitor(id).expect(&format!(
            "Could not get monitor for monitor index {:?}",
            id
        ));
        Self::new(id, mon)
    }

    pub fn all() -> Vec<Self> {
        let default_display = gdk::Display::get_default()
            .expect("gdk should always find a Display when it runs");
        let num_monitors = default_display.get_n_monitors();
        (0..num_monitors)
            .map(|monitor_index| {
                Self::new_from_id(&default_display, monitor_index)
            })
            .collect()
    }
}

impl std::ops::Deref for Monitor {
    type Target = gdk::Monitor;

    fn deref(&self) -> &Self::Target {
        &self.monitor
    }
}

impl AsRef<gdk::Monitor> for Monitor {
    fn as_ref(&self) -> &gdk::Monitor {
        &*self
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub builders: Vec<gtk::Builder>,
    pub monitors: Vec<Monitor>,
    pub sender: glib::Sender<Message>,
    pub presses_remaining: Arc<RwLock<u32>>,
    pub start_time: SystemTime,
    pub app_sender: glib::Sender<Msg>,
}

impl State {
    pub fn new(
        config: &Config,
        app_sender: glib::Sender<Msg>,
        sender: glib::Sender<Message>,
    ) -> Self {
        let monitors = Monitor::all();
        let monitors_num = monitors.len();

        let builders = std::iter::repeat_with(builder::create)
            .take(monitors_num)
            .collect();

        Self {
            builders,
            monitors,
            sender,
            presses_remaining: Arc::new(RwLock::new(
                config.settings.clicks_to_end_break_early,
            )),
            start_time: SystemTime::now(),
            app_sender,
        }
    }

    pub fn read_presses_remaining(&self) -> RwLockReadGuard<u32> {
        self.presses_remaining.read().unwrap()
    }

    /// Decrements the number of presses remaining by 1.
    pub fn decrement_presses_remaining(&self) -> u32 {
        let state_presses_remaining: &mut u32 =
            &mut *self.presses_remaining.write().unwrap();

        if *state_presses_remaining > 0 {
            *state_presses_remaining -= 1;
        }

        *state_presses_remaining
    }

    pub fn get_app_wins(&self) -> Vec<gtk::ApplicationWindow> {
        self.builders
            .iter()
            .map(|builder| builder.get_object_expect("app_win"))
            .collect()
    }

    pub fn get_app_wins_with_monitors(
        &self,
    ) -> Vec<(gtk::ApplicationWindow, Monitor)> {
        self.builders
            .iter()
            .zip(&self.monitors)
            .map(|(builder, monitor)| {
                (builder.get_object_expect("app_win"), monitor.clone())
            })
            .collect()
    }

    pub fn get_time_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders
            .iter()
            .map(|builder| builder.get_object_expect("time_remaining_label"))
            .collect()
    }

    pub fn get_presses_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders
            .iter()
            .map(|builder| builder.get_object_expect("presses_remaining_label"))
            .collect()
    }

    pub fn notify_app_end(&self) {
        self.app_sender.send(Msg::EndBreak).expect(
            "TODO: figure out what to do about channels potentially failing",
        );
    }

    pub fn end(&self) {
        self.sender.send(Message::End).expect(
            "TODO: figure out what to do about channels potentially failing",
        );
    }
}
