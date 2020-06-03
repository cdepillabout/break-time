#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod prelude;
mod scheduler;
mod tray;
pub mod ui;
mod x11;

use std::sync::mpsc::Sender;

use config::Config;
use prelude::*;
use scheduler::Scheduler;
use tray::Tray;

pub enum Msg {
    EndBreak,
    Quit,
    TimeRemainingBeforeBreak,
    StartBreak,
}

fn handle_msg_recv(
    sender: glib::Sender<Msg>,
    scheduler_sender: Sender<scheduler::Msg>,
    tray: &Tray,
    msg: Msg,
) {
    match msg {
        Msg::EndBreak => {
            println!("break ended");
            scheduler_sender.send(scheduler::Msg::Start);
        }
        Msg::Quit => {
            gtk::main_quit();
        }
        Msg::StartBreak => {
            println!("starting break");
            ui::start_break(sender);
        }
        Msg::TimeRemainingBeforeBreak => {
            tray.render_time_remaining_before_break();
        }
    }
}

pub fn default_main() {
    let config = Config::load().expect("Could not load config file.");

    gtk::init().expect("Could not initialize GTK");

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    // TODO: pass the tray to the scheduler so the scheduler can determine when to start counting
    // down on the tray...
    let tray = tray::Tray::run(sender.clone());

    println!("Starting the scheduler...");
    let scheduler_sender = Scheduler::run(config, sender.clone());

    receiver.attach(None, move |msg| {
        handle_msg_recv(sender.clone(), scheduler_sender.clone(), &tray, msg);
        glib::source::Continue(true)
    });

    gtk::main();
}
