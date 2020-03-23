#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod scheduler;
mod prelude;
mod tray;
pub mod ui;

use scheduler::Scheduler;
use prelude::*;


pub enum Msg {
    StartBreak,
    EndBreak,
}

fn handle_msg_recv(sender: glib::Sender<Msg>, scheduler: Scheduler, msg: Msg) {
    match msg {
        Msg::StartBreak => {
            println!("starting break");
            ui::start_break(sender);
        }
        Msg::EndBreak => {
            println!("break ended");
            scheduler.run();
        }
    }
}

pub fn default_main() {
    gtk::init().expect("Could not initialize GTK");

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let scheduler = Scheduler::new(sender.clone()).expect("Couldn't create a scheduler!");

    scheduler.run();

    tray::show();

    receiver.attach(
        None,
        move |msg| {
            handle_msg_recv(sender.clone(), scheduler.clone(), msg);
            glib::source::Continue(true)
        },
    );

    gtk::main();
}
