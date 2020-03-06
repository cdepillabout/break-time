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
    pub sender: glib::Sender<Message>,
    pub presses_remaining: Arc<RwLock<u32>>,
    pub start_instant: Instant,
}

impl State {
    pub fn new(app: gtk::Application, sender: glib::Sender<Message>, monitors: usize) -> Self {
        State {
            app,
            builders: std::iter::repeat_with(builder::create).take(monitors).collect(),
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

    pub fn get_time_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders.iter().map(|builder| builder.get_object_expect("time_remaining_label")).collect()
    }

    pub fn get_presses_remaining_labels(&self) -> Vec<gtk::Label> {
        self.builders.iter().map(|builder| builder.get_object_expect("presses_remaining_label")).collect()
    }
}
