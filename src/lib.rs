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
}

fn handle_msg_recv(msg: Msg) {
    match msg {
        Msg::StartBreak => {
            println!("starting break");
            ui::start_break();
        }
    }
}

pub fn default_main() {
    gtk::init().expect("Could not initialize GTK");

    let (sender, receiver) =
        glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

    let scheduler = Scheduler::new(sender);

    scheduler.run();

    tray::show();

    receiver.attach(
        None,
        |msg| {
            handle_msg_recv(msg);
            glib::source::Continue(true)
        },
    );

    gtk::main();
}
