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

pub enum Msg {
    EndBreak,
    Quit,
    StartBreak,
}

fn handle_msg_recv(
    sender: glib::Sender<Msg>,
    scheduler_sender: Sender<scheduler::Msg>,
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
    }
}

pub fn default_main() {
    let config = Config::load().expect("Could not load config file.");

    gtk::init().expect("Could not initialize GTK");

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    // let scheduler = Scheduler::new(sender.clone()).expect("Couldn't create a scheduler!");

    // scheduler.run();

    println!("Starting the scheduler...");
    let scheduler_sender = Scheduler::run(config, sender.clone());

    tray::run(sender.clone());

    receiver.attach(None, move |msg| {
        handle_msg_recv(sender.clone(), scheduler_sender.clone(), msg);
        glib::source::Continue(true)
    });

    gtk::main();
}
