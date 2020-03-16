#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

mod scheduler;
mod prelude;
mod tray;
pub mod ui;

use scheduler::Scheduler;
use prelude::*;

// fn app_activate(app: gtk::Application) {

//     let (sender, receiver) =
//         glib::MainContext::channel(glib::source::PRIORITY_DEFAULT);

//     let state = State::new(app, sender);

//     setup(&state);

//     connect_events(&state);

//     redisplay(&state);

//     for (window, monitor) in state.get_app_wins_with_monitors() {
//         window.show_all();

//         let monitor_rect = monitor.get_geometry();
//         let gdk_window: gdk::Window = window.get_window().expect(
//             "Gtk::Window should always be able to be converted to Gdk::Window",
//         );
//         gdk_window.fullscreen_on_monitor(monitor.id);
//         // gdk_window.resize(monitor_rect.width, monitor_rect.height);
//     }

//     receiver.attach(
//         None,
//         clone!(@strong state => move |msg| {
//             handle_msg_recv(&state, msg);
//             glib::source::Continue(true)
//         }),
//     );
    

// }

pub enum Msg {
    StartBreak,
}

fn handle_msg_recv(msg: Msg) {
    match msg {
        Msg::StartBreak => {
            println!("starting break");
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
