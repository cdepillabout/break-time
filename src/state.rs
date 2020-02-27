use super::builder;
use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Display(ExecNixStoreRes),
}

#[derive(Clone, Debug)]
pub struct State {
    pub app: gtk::Application,
    pub builder: gtk::Builder,
    pub sender: glib::Sender<Message>,
    pub presses_remaining: Arc<RwLock<u32>>,
}

impl State {
    pub fn new(app: gtk::Application, sender: glib::Sender<Message>) -> Self {
        State {
            app,
            builder: builder::create(),
            sender,
            presses_remaining: Arc::new(RwLock::new(2)),
        }
    }

    pub fn read_presses_remaining(&self) -> RwLockReadGuard<u32> {
        self.presses_remaining.read().unwrap()
    }

    pub fn write_presses_remaining(&self, new_presses_remaining: u32) {
        let state_presses_remaining: &mut u32 =
            &mut *self.presses_remaining.write().unwrap();
        *state_option_nix_store_res = Some(new_nix_store_res);
    }

    pub fn get_app_win(&self) -> gtk::ApplicationWindow {
        self.builder.get_object_expect("app_win")
    }

    pub fn get_time_remaining_label(&self) -> gtk::Label {
        self.builder.get_object_expect("time_remaining_label")
    }

    pub fn get_presses_remaining_label(&self) -> gtk::Label {
        self.builder.get_object_expect("presses_remaining_label")
    }
}
