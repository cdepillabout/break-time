use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Instant;
use super::builder;
use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Display,
}

#[derive(Clone, Debug)]
pub struct State {
    pub app: gtk::Application,
    pub builders: Vec<gtk::Builder>,
    pub monitors: Vec<gdk::Monitor>,
    pub sender: glib::Sender<Message>,
    pub presses_remaining: Arc<RwLock<u32>>,
    pub start_instant: Instant,
}

impl State {
    pub fn new(app: gtk::Application, sender: glib::Sender<Message>) -> Self {

        let default_display = gdk::Display::get_default().expect("gtk should always find a Display when it runs");
        let num_monitors = default_display.get_n_monitors();
        let monitors = (0..num_monitors).map(|monitor_index| default_display.get_monitor(monitor_index).expect(&format!("Could not get monitor for monitor index {:?}", monitor_index))).collect();
        let num_monitors_usize = usize::try_from(num_monitors).unwrap_or(0);

        let builders = std::iter::repeat_with(builder::create).take(num_monitors_usize).collect();

        State {
            app,
            builders,
            monitors,
            sender,
            presses_remaining: Arc::new(RwLock::new(2)),
            start_instant: Instant::now(),
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
            *state_presses_remaining = *state_presses_remaining - 1;
        }

        *state_presses_remaining
    }

    pub fn get_app_wins(&self) -> Vec<gtk::ApplicationWindow> {
        self.builders.iter().map(|builder| builder.get_object_expect("app_win")).collect()
    }

    pub fn get_app_wins_with_monitors(&self) -> Vec<(gtk::ApplicationWindow, gdk::Monitor)> {
        self.builders.iter().zip(&self.monitors).map(|(builder, monitor)| {
            (builder.get_object_expect("app_win"), monitor.clone())
        }).collect()
    }

    pub fn get_time_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders.iter().map(|builder| builder.get_object_expect("time_remaining_label")).collect()
    }

    pub fn get_presses_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders.iter().map(|builder| builder.get_object_expect("presses_remaining_label")).collect()
    }
}
